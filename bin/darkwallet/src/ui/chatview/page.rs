/* This file is part of DarkFi (https://dark.fi)
 *
 * Copyright (C) 2020-2024 Dyne.org foundation
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Affero General Public License as
 * published by the Free Software Foundation, either version 3 of the
 * License, or (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU Affero General Public License for more details.
 *
 * You should have received a copy of the GNU Affero General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

use async_gen::{gen as async_gen, AsyncIter};
use async_lock::Mutex as AsyncMutex;
use chrono::{Local, NaiveDate, TimeZone};
use futures::stream::{Stream, StreamExt};
use std::{
    collections::HashMap,
    hash::{DefaultHasher, Hash, Hasher},
    io::Cursor,
    pin::pin,
    sync::{atomic::Ordering, Arc, Mutex as SyncMutex, Weak},
};

use super::{max, MessageId, Timestamp};
use crate::{
    gfx::{
        GfxBufferId, GfxDrawCall, GfxDrawInstruction, GfxDrawMesh, GfxTextureId,
        GraphicsEventPublisherPtr, Point, Rectangle, RenderApi, RenderApiPtr,
    },
    mesh::{Color, MeshBuilder, COLOR_BLUE, COLOR_GREEN, COLOR_PINK},
    prop::{PropertyBool, PropertyColor, PropertyFloat32, PropertyPtr, PropertyUint32, Role},
    pubsub::Subscription,
    text::{self, glyph_str, Glyph, GlyphPositionIter, TextShaper, TextShaperPtr},
    ui::FreedData,
    util::{enumerate_mut, enumerate_ref},
    ExecutorPtr,
};

const PAGE_SIZE: usize = 10;
const PRELOAD_PAGES: usize = 10;

fn is_whitespace(s: &str) -> bool {
    s.chars().all(char::is_whitespace)
}

#[derive(Clone)]
pub struct PrivMessage {
    font_size: f32,
    timestamp_font_size: f32,
    window_scale: f32,

    timestamp: Timestamp,
    id: MessageId,
    nick: String,
    text: String,

    is_selected: bool,

    time_glyphs: Vec<Glyph>,
    unwrapped_glyphs: Vec<Glyph>,
    wrapped_lines: Vec<Vec<Glyph>>,

    atlas: text::RenderedAtlas,
    mesh_cache: Option<GfxDrawMesh>,
}

impl PrivMessage {
    pub async fn new(
        font_size: f32,
        timestamp_font_size: f32,
        window_scale: f32,

        timestamp: Timestamp,
        id: MessageId,
        nick: String,
        text: String,

        line_width: f32,
        timestamp_width: f32,

        text_shaper: &TextShaper,
        render_api: &RenderApi,
    ) -> Message {
        let timestr = Self::gen_timestr(timestamp);
        let time_glyphs = text_shaper.shape(timestr, timestamp_font_size, window_scale).await;

        let linetext = format!("{nick} {text}");
        let unwrapped_glyphs = text_shaper.shape(linetext, font_size, window_scale).await;

        let mut atlas = text::Atlas::new(render_api);
        atlas.push(&time_glyphs);
        atlas.push(&unwrapped_glyphs);
        let atlas = atlas.make();

        let mut self_ = Self {
            font_size,
            timestamp_font_size,
            window_scale,
            timestamp,
            id,
            nick,
            text,
            is_selected: false,
            time_glyphs,
            unwrapped_glyphs,
            wrapped_lines: vec![],
            atlas,
            mesh_cache: None,
        };
        self_.adjust_width(line_width, timestamp_width);
        Message::Priv(self_)
    }

    fn gen_timestr(timestamp: Timestamp) -> String {
        let dt = Local.timestamp_millis_opt(timestamp as i64).unwrap();
        let timestr = dt.format("%H:%M").to_string();
        timestr
    }

    fn height(&self, line_height: f32) -> f32 {
        self.wrapped_lines.len() as f32 * line_height
    }

    fn gen_mesh(
        &mut self,
        clip: &Rectangle,
        line_height: f32,
        baseline: f32,
        timestamp_width: f32,
        nick_colors: &[Color],
        timestamp_color: Color,
        text_color: Color,
        hi_bg_color: Color,
        debug_render: bool,
        render_api: &RenderApi,
    ) -> GfxDrawMesh {
        if let Some(mesh) = &self.mesh_cache {
            return mesh.clone()
        }

        //debug!(target: "ui::chatview", "gen_mesh({})", glyph_str(&self.unwrapped_glyphs));
        let mut mesh = MeshBuilder::new();

        if self.is_selected {
            let height = self.height(line_height);
            mesh.draw_filled_box(
                &Rectangle { x: 0., y: -height, w: clip.w, h: height },
                hi_bg_color,
            );
        }

        self.render_timestamp(&mut mesh, baseline, line_height, timestamp_color);
        let off_x = timestamp_width;

        let nick_color = select_nick_color(&self.nick, nick_colors);

        let last_idx = self.wrapped_lines.len() - 1;
        for (i, line) in self.wrapped_lines.iter().rev().enumerate() {
            let off_y = (i + 1) as f32 * line_height;
            let is_last_line = i == last_idx;

            // debug draw baseline
            if debug_render {
                let y = baseline - off_y;
                mesh.draw_filled_box(&Rectangle { x: 0., y: y - 1., w: clip.w, h: 1. }, COLOR_BLUE);
            }

            self.render_line(
                &mut mesh,
                line,
                off_x,
                off_y,
                is_last_line,
                baseline,
                nick_color,
                text_color,
                debug_render,
            );
        }

        if debug_render {
            let height = self.height(line_height);
            mesh.draw_outline(
                &Rectangle { x: 0., y: -height, w: clip.w, h: height },
                COLOR_PINK,
                1.,
            );
        }

        let mesh = mesh.alloc(render_api);
        let mesh = mesh.draw_with_texture(self.atlas.texture_id);
        self.mesh_cache = Some(mesh.clone());

        mesh
    }

    fn render_timestamp(&self, mesh: &mut MeshBuilder, baseline: f32, line_height: f32, timestamp_color: Color) {
        let off_y = self.wrapped_lines.len() as f32 * line_height;

        let glyph_pos_iter = GlyphPositionIter::new(
            self.timestamp_font_size,
            self.window_scale,
            &self.time_glyphs,
            baseline,
        );
        for (mut glyph_rect, glyph) in glyph_pos_iter.zip(self.time_glyphs.iter()) {
            let uv_rect = self.atlas.fetch_uv(glyph.glyph_id).expect("missing glyph UV rect");
            glyph_rect.y -= off_y;

            mesh.draw_box(&glyph_rect, timestamp_color, uv_rect);
        }
    }

    fn render_line(
        &self,
        mesh: &mut MeshBuilder,
        line: &Vec<Glyph>,
        off_x: f32,
        off_y: f32,
        is_last: bool,
        baseline: f32,
        nick_color: Color,
        text_color: Color,
        debug_render: bool,
    ) {
        //debug!(target: "ui::chatview", "render_line({})", glyph_str(line));
        // Keep track of the 'section'
        // Section 0   is the nickname (colorized)
        // Section >=1 is just the message itself
        let mut section = 1;
        if is_last {
            section = 0;
        }

        let glyph_pos_iter =
            GlyphPositionIter::new(self.font_size, self.window_scale, line, baseline);
        for (mut glyph_rect, glyph) in glyph_pos_iter.zip(line.iter()) {
            let uv_rect = self.atlas.fetch_uv(glyph.glyph_id).expect("missing glyph UV rect");

            glyph_rect.x += off_x;
            glyph_rect.y -= off_y;

            let color = match section {
                0 => nick_color,
                _ => text_color,
            };

            //if debug_render {
            //    mesh.draw_outline(&glyph_rect, COLOR_BLUE, 2.);
            //}

            mesh.draw_box(&glyph_rect, color, uv_rect);

            if is_last && section < 1 && is_whitespace(&glyph.substr) {
                section += 1;
            }
        }
    }

    /// clear_mesh() must be called after this.
    async fn adjust_params(
        &mut self,
        font_size: f32,
        timestamp_font_size: f32,
        window_scale: f32,
        line_width: f32,
        timestamp_width: f32,
        text_shaper: &TextShaper,
        render_api: &RenderApi,
    ) -> GfxTextureId {
        self.font_size = font_size;
        self.timestamp_font_size = timestamp_font_size;
        self.window_scale = window_scale;

        let timestr = Self::gen_timestr(self.timestamp);
        self.time_glyphs = text_shaper.shape(timestr, timestamp_font_size, window_scale).await;

        let linetext = format!("{} {}", self.nick, self.text);
        self.unwrapped_glyphs = text_shaper.shape(linetext, font_size, window_scale).await;

        let texture_id = self.atlas.texture_id;

        let mut atlas = text::Atlas::new(render_api);
        atlas.push(&self.time_glyphs);
        atlas.push(&self.unwrapped_glyphs);
        self.atlas = atlas.make();

        // We need to rewrap the glyphs since they've been reloaded
        self.adjust_width(line_width, timestamp_width);

        texture_id
    }

    /// clear_mesh() must be called after this.
    fn adjust_width(&mut self, line_width: f32, timestamp_width: f32) {
        let width = line_width - timestamp_width;
        // clamp to > 0
        let width = max(width, 0.);

        // Invalidate wrapped_glyphs and recalc
        self.wrapped_lines =
            text::wrap(width, self.font_size, self.window_scale, &self.unwrapped_glyphs);
    }

    fn clear_mesh(&mut self) -> Option<GfxDrawMesh> {
        std::mem::replace(&mut self.mesh_cache, None)
    }

    fn select(&mut self) {
        self.is_selected = true;
    }
}

impl std::fmt::Debug for PrivMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let dt = Local.timestamp_millis_opt(self.timestamp as i64).unwrap();
        let timestr = dt.format("%H:%M").to_string();
        write!(f, "{} <{}> {}", timestr, self.nick, self.text)
    }
}

#[derive(Clone)]
pub struct DateMessage {
    font_size: f32,
    window_scale: f32,

    timestamp: Timestamp,
    glyphs: Vec<Glyph>,

    atlas: text::RenderedAtlas,
    mesh_cache: Option<GfxDrawMesh>,
}

impl DateMessage {
    pub async fn new(
        font_size: f32,
        window_scale: f32,

        timestamp: Timestamp,

        text_shaper: &TextShaper,
        render_api: &RenderApi,
    ) -> Message {
        let datestr = Self::datestr(timestamp);
        let timestamp = Self::timest_to_midnight(timestamp);

        let glyphs = text_shaper.shape(datestr, font_size, window_scale).await;

        let mut atlas = text::Atlas::new(render_api);
        atlas.push(&glyphs);
        let atlas = atlas.make();

        Message::Date(Self { font_size, window_scale, timestamp, glyphs, atlas, mesh_cache: None })
    }

    fn datestr(timestamp: Timestamp) -> String {
        let dt = Local.timestamp_millis_opt(timestamp as i64).unwrap();
        let datestr = dt.format("%a %-d %b %Y").to_string();
        datestr
    }

    fn timest_to_midnight(timestamp: Timestamp) -> Timestamp {
        let dt = Local.timestamp_millis_opt(timestamp as i64).unwrap();
        let dt2 = dt.date_naive().and_hms_opt(0, 0, 0).unwrap();
        assert_eq!(dt.date_naive(), dt2.date());
        let timestamp = Local.from_local_datetime(&dt2).unwrap().timestamp_millis() as u64;
        timestamp
    }

    /// clear_mesh() must be called after this.
    async fn adjust_params(
        &mut self,
        font_size: f32,
        window_scale: f32,
        text_shaper: &TextShaper,
        render_api: &RenderApi,
    ) -> GfxTextureId {
        self.font_size = font_size;
        self.window_scale = window_scale;

        let datestr = Self::datestr(self.timestamp);
        self.glyphs = text_shaper.shape(datestr, font_size, window_scale).await;

        let texture_id = self.atlas.texture_id;

        let mut atlas = text::Atlas::new(render_api);
        atlas.push(&self.glyphs);
        self.atlas = atlas.make();

        texture_id
    }

    //fn adjust_width(&mut self, line_width: f32) { }

    fn clear_mesh(&mut self) -> Option<GfxDrawMesh> {
        std::mem::replace(&mut self.mesh_cache, None)
    }

    fn gen_mesh(
        &mut self,
        clip: &Rectangle,
        line_height: f32,
        baseline: f32,
        nick_colors: &[Color],
        timestamp_color: Color,
        text_color: Color,
        debug_render: bool,
        render_api: &RenderApi,
    ) -> GfxDrawMesh {
        let mut mesh = MeshBuilder::new();

        let glyph_pos_iter =
            GlyphPositionIter::new(self.font_size, self.window_scale, &self.glyphs, baseline);
        for (mut glyph_rect, glyph) in glyph_pos_iter.zip(self.glyphs.iter()) {
            let uv_rect = self.atlas.fetch_uv(glyph.glyph_id).expect("missing glyph UV rect");
            glyph_rect.y -= line_height;
            mesh.draw_box(&glyph_rect, timestamp_color, uv_rect);
        }

        if debug_render {
            mesh.draw_outline(
                &Rectangle { x: 0., y: -line_height, w: clip.w, h: line_height },
                COLOR_PINK,
                1.,
            );
        }

        let mesh = mesh.alloc(render_api);
        let mesh = mesh.draw_with_texture(self.atlas.texture_id);
        self.mesh_cache = Some(mesh.clone());

        mesh
    }
}

impl std::fmt::Debug for DateMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let dt = Local.timestamp_millis_opt(self.timestamp as i64).unwrap();
        let datestr = dt.format("%a %-d %b %Y").to_string();
        write!(f, "{}", datestr)
    }
}

/// Easier than fucking around with traits nonsense
#[derive(Debug)]
enum Message {
    Priv(PrivMessage),
    Date(DateMessage),
}

impl Message {
    fn timestamp(&self) -> u64 {
        match self {
            Self::Priv(m) => m.timestamp,
            Self::Date(m) => m.timestamp,
        }
    }

    fn height(&self, line_height: f32) -> f32 {
        match self {
            Self::Priv(m) => m.height(line_height),
            Self::Date(_) => line_height,
        }
    }

    async fn adjust_params(
        &mut self,
        font_size: f32,
        timestamp_font_size: f32,
        window_scale: f32,
        line_width: f32,
        timestamp_width: f32,
        text_shaper: &TextShaper,
        render_api: &RenderApi,
    ) -> GfxTextureId {
        match self {
            Self::Priv(m) => {
                m.adjust_params(
                    font_size,
                    timestamp_font_size,
                    window_scale,
                    line_width,
                    timestamp_width,
                    text_shaper,
                    render_api,
                )
                .await
            }
            Self::Date(m) => {
                m.adjust_params(font_size, window_scale, text_shaper, render_api).await
            }
        }
    }

    fn adjust_width(&mut self, line_width: f32, timestamp_width: f32) {
        match self {
            Self::Priv(m) => m.adjust_width(line_width, timestamp_width),
            Self::Date(_) => {}
        }
    }

    fn clear_mesh(&mut self) -> Option<GfxDrawMesh> {
        match self {
            Self::Priv(m) => m.clear_mesh(),
            Self::Date(m) => m.clear_mesh(),
        }
    }

    fn gen_mesh(
        &mut self,
        clip: &Rectangle,
        line_height: f32,
        baseline: f32,
        timestamp_width: f32,
        nick_colors: &[Color],
        timestamp_color: Color,
        text_color: Color,
        hi_bg_color: Color,
        debug_render: bool,
        render_api: &RenderApi,
    ) -> GfxDrawMesh {
        match self {
            Self::Priv(m) => m.gen_mesh(
                clip,
                line_height,
                baseline,
                timestamp_width,
                nick_colors,
                timestamp_color,
                text_color,
                hi_bg_color,
                debug_render,
                render_api,
            ),
            Self::Date(m) => m.gen_mesh(
                clip,
                line_height,
                baseline,
                // No timestamp_width
                nick_colors,
                timestamp_color,
                text_color,
                // No hi_bg_color since dates can't be highlighted
                debug_render,
                render_api,
            ),
        }
    }

    fn is_date(&self) -> bool {
        match self {
            Self::Priv(m) => false,
            Self::Date(_) => true,
        }
    }

    fn select(&mut self) {
        match self {
            Self::Priv(m) => m.select(),
            Self::Date(_) => {}
        }
    }
}

fn select_nick_color(nick: &str, nick_colors: &[Color]) -> Color {
    let mut hasher = DefaultHasher::new();
    nick.hash(&mut hasher);
    let i = hasher.finish() as usize;
    let color = nick_colors[i % nick_colors.len()];
    color
}

pub struct MessageBuffer {
    /// From most recent to older
    msgs: Vec<Message>,
    date_msgs: HashMap<NaiveDate, Message>,
    pub freed: FreedData,
    pub line_width: f32,

    font_size: PropertyFloat32,
    timestamp_font_size: PropertyFloat32,
    timestamp_width: PropertyFloat32,
    line_height: PropertyFloat32,
    msg_spacing: PropertyFloat32,
    baseline: PropertyFloat32,
    timestamp_color: PropertyColor,
    text_color: PropertyColor,
    nick_colors: PropertyPtr,
    hi_bg_color: PropertyColor,
    debug: PropertyBool,

    window_scale: PropertyFloat32,
    /// Used to detect if the window scale was changed when drawing.
    /// If it does then we must reload the glyphs too.
    old_window_scale: f32,

    render_api: RenderApiPtr,
    text_shaper: TextShaperPtr,
}

impl MessageBuffer {
    pub fn new(
        font_size: PropertyFloat32,
        timestamp_font_size: PropertyFloat32,
        timestamp_width: PropertyFloat32,
        line_height: PropertyFloat32,
        msg_spacing: PropertyFloat32,
        baseline: PropertyFloat32,
        timestamp_color: PropertyColor,
        text_color: PropertyColor,
        nick_colors: PropertyPtr,
        hi_bg_color: PropertyColor,
        debug: PropertyBool,
        window_scale: PropertyFloat32,
        render_api: RenderApiPtr,
        text_shaper: TextShaperPtr,
    ) -> Self {
        let old_window_scale = window_scale.get();
        Self {
            msgs: vec![],
            date_msgs: HashMap::new(),
            freed: Default::default(),
            line_width: 0.,

            font_size,
            timestamp_font_size,
            timestamp_width,
            line_height,
            msg_spacing,
            baseline,
            timestamp_color,
            text_color,
            nick_colors,
            hi_bg_color,
            debug,

            window_scale,
            old_window_scale,

            render_api,
            text_shaper,
        }
    }

    pub async fn adjust_window_scale(&mut self) {
        let window_scale = self.window_scale.get();
        if self.old_window_scale == window_scale {
            return
        }

        self.adjust_params().await;
    }

    /// This will force a reload of everything
    pub async fn adjust_params(&mut self) {
        let window_scale = self.window_scale.get();
        let font_size = self.font_size.get();
        let timestamp_font_size = self.timestamp_font_size.get();
        let timestamp_width = self.timestamp_width.get();

        for msg in &mut self.msgs {
            let old_texture_id = msg
                .adjust_params(
                    font_size,
                    timestamp_font_size,
                    window_scale,
                    self.line_width,
                    timestamp_width,
                    &self.text_shaper,
                    &self.render_api,
                )
                .await;
            self.freed.add_texture(old_texture_id);
        }
    }

    /// For scrolling we want to be able to adjust and measure without
    /// explicitly rendering since it may be off screen.
    pub fn adjust_width(&mut self, line_width: f32) {
        if (line_width - self.line_width).abs() < f32::EPSILON {
            return;
        }
        self.line_width = line_width;

        let timestamp_width = self.timestamp_width.get();

        for msg in &mut self.msgs {
            msg.adjust_width(line_width, timestamp_width);
        }
    }

    /// Clear all meshes and caches. Returns data that needs to be freed.
    pub fn clear_meshes(&mut self) {
        for msg in &mut self.msgs {
            if let Some(mesh) = msg.clear_mesh() {
                self.freed.add_mesh(mesh);
            }
        }
    }

    pub async fn calc_total_height(&mut self) -> f32 {
        let line_height = self.line_height.get();
        let msg_spacing = self.msg_spacing.get();
        let mut height = 0.;

        let msgs = self.msgs_with_date();
        let mut msgs = pin!(msgs);

        let mut is_first = true;

        while let Some(msg) = msgs.next().await {
            if is_first {
                is_first = false;
                height += msg_spacing;
            }

            height += msg.height(line_height);
        }

        height
    }

    pub async fn insert_privmsg(
        &mut self,
        timest: Timestamp,
        msg_id: MessageId,
        nick: String,
        text: String,
    ) {
        //debug!(target: "ui::chatview", "MessageBuffer::insert_privmsg()");
        let font_size = self.font_size.get();
        let timestamp_font_size = self.timestamp_font_size.get();
        let timestamp_width = self.timestamp_width.get();
        let window_scale = self.window_scale.get();

        let msg = PrivMessage::new(
            font_size,
            timestamp_font_size,
            window_scale,
            timest,
            msg_id,
            nick,
            text,
            self.line_width,
            timestamp_width,
            &self.text_shaper,
            &self.render_api,
        )
        .await;

        if self.msgs.is_empty() {
            self.msgs.push(msg);
            return
        }

        // We only add lines inside pages.
        // Calling the appropriate draw() function after should preload any missing pages.
        // When a line is before the first page, it will get preloaded as a new page.
        let oldest_timest = self.oldest_timestamp().unwrap();
        if timest < oldest_timest {
            return;
        }

        // Timestamps go from most recent backwards

        let mut idx = None;
        for (i, msg) in enumerate_mut(&mut self.msgs) {
            if timest >= msg.timestamp() {
                idx = Some(i);
                break
            }
        }

        let idx = match idx {
            Some(idx) => idx,
            None => {
                let last_page_idx = 0;
                last_page_idx
            }
        };

        self.msgs.insert(idx, msg);
    }

    pub async fn push_privmsg(
        &mut self,
        timest: Timestamp,
        msg_id: MessageId,
        nick: String,
        text: String,
    ) -> f32 {
        let font_size = self.font_size.get();
        let timestamp_font_size = self.timestamp_font_size.get();
        let timestamp_width = self.timestamp_width.get();
        let window_scale = self.window_scale.get();

        let msg = PrivMessage::new(
            font_size,
            timestamp_font_size,
            window_scale,
            timest,
            msg_id,
            nick,
            text,
            self.line_width,
            timestamp_width,
            &self.text_shaper,
            &self.render_api,
        )
        .await;

        let msg_height = msg.height(self.line_height.get());

        if self.msgs.is_empty() {
            self.msgs.push(msg);
            return msg_height
        }

        self.msgs.push(msg);
        msg_height
    }

    /// Generate caches and return meshes
    pub async fn gen_meshes(&mut self, rect: &Rectangle, scroll: f32) -> Vec<(f32, GfxDrawMesh)> {
        let line_height = self.line_height.get();
        let msg_spacing = self.msg_spacing.get();
        let baseline = self.baseline.get();
        let timestamp_width = self.timestamp_width.get();
        let debug_render = self.debug.get();

        let timest_color = self.timestamp_color.get();
        let text_color = self.text_color.get();
        let nick_colors = self.read_nick_colors();
        let hi_bg_color = self.hi_bg_color.get();

        let render_api = self.render_api.clone();

        let msgs = self.msgs_with_date();
        let mut msgs = pin!(msgs);

        let mut meshes = vec![];
        let mut current_pos = 0.;
        while let Some(msg) = msgs.next().await {
            let mesh_height = msg.height(line_height);
            let msg_bottom = current_pos;
            let msg_top = current_pos + mesh_height;

            if msg_bottom > scroll + rect.h {
                break
            }
            if msg_top < scroll {
                current_pos += mesh_height;
                continue
            }

            let mesh = msg.gen_mesh(
                rect,
                line_height,
                baseline,
                timestamp_width,
                &nick_colors,
                timest_color,
                text_color,
                hi_bg_color,
                debug_render,
                &render_api,
            );

            meshes.push((current_pos, mesh));

            current_pos += msg_spacing;
            current_pos += mesh_height;
        }

        //debug!("gen_meshes() returning {} meshes", meshes.len());
        meshes
    }

    /// Gets around borrow checker with unsafe
    fn msgs_with_date(&mut self) -> impl Stream<Item = &mut Message> {
        let font_size = self.font_size.get();
        let window_scale = self.window_scale.get();
        AsyncIter::from(async_gen! {
            let mut last_date = None;

            for idx in 0..self.msgs.len() {
                let msg = &mut self.msgs[idx] as *mut Message;
                let msg = unsafe { &mut *msg };
                let timest = msg.timestamp();

                let older_date = Local.timestamp_millis_opt(timest as i64).unwrap().date_naive();

                if let Some(newer_date) = last_date {
                    if newer_date != older_date {
                        let datemsg = self.get_date_msg(newer_date, font_size, window_scale).await;
                        let datemsg = unsafe { &mut *(datemsg as *mut Message) };
                        //debug!(target: "ui::chatview", "Adding date: {idx} {datemsg:?}");
                        yield datemsg;
                    }
                }
                last_date = Some(older_date);

                //debug!(target: "ui::chatview", "{idx} {msg:?}");
                yield msg;
            }

            if let Some(date) = last_date {
                let datemsg = self.get_date_msg(date, font_size, window_scale).await;
                let datemsg = unsafe { &mut *(datemsg as *mut Message) };
                yield datemsg;
            }
        })
    }

    async fn get_date_msg(
        &mut self,
        date: NaiveDate,
        font_size: f32,
        window_scale: f32,
    ) -> &mut Message {
        let dt = date.and_hms_opt(0, 0, 0).unwrap();
        let timest = Local.from_local_datetime(&dt).unwrap().timestamp_millis() as u64;

        if !self.date_msgs.contains_key(&date) {
            let datemsg = DateMessage::new(
                font_size,
                window_scale,
                timest,
                &self.text_shaper,
                &self.render_api,
            )
            .await;
            self.date_msgs.insert(date, datemsg);
        }

        self.date_msgs.get_mut(&date).unwrap()
    }

    pub fn oldest_timestamp(&self) -> Option<Timestamp> {
        let last_msg = &self.msgs.last()?;
        Some(last_msg.timestamp())
    }

    pub fn latest_timestamp(&self) -> Option<Timestamp> {
        let first_msg = &self.msgs.first()?;
        Some(first_msg.timestamp())
    }

    fn read_nick_colors(&self) -> Vec<Color> {
        let mut colors = vec![];
        let mut color = [0f32; 4];
        for i in 0..self.nick_colors.get_len() {
            color[i % 4] = self.nick_colors.get_f32(i).expect("prop logic err");

            if i > 0 && i % 4 == 0 {
                let color = std::mem::take(&mut color);
                colors.push(color);
            }
        }
        colors
    }

    pub async fn select_line(&mut self, y: f32) {
        let line_height = self.line_height.get();
        let msg_spacing = self.msg_spacing.get();

        let msgs = self.msgs_with_date();
        let mut msgs = pin!(msgs);

        let mut current_pos = 0.;
        while let Some(msg) = msgs.next().await {
            let mesh_height = msg.height(line_height);
            let msg_bottom = current_pos;
            let msg_top = current_pos + mesh_height;

            if msg_bottom <= y && y <= msg_top {
                // Do nothing
                if msg.is_date() {
                    break
                }

                msg.select();
                msg.clear_mesh();
                break
            }

            current_pos += msg_spacing;
            current_pos += mesh_height;
        }
    }
}
