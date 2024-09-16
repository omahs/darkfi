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

use async_recursion::async_recursion;
use async_trait::async_trait;
use miniquad::{KeyCode, KeyMods, MouseButton, TouchPhase};
use rand::{rngs::OsRng, Rng};
use std::sync::{Arc, Weak};

use crate::{
    gfx::{GfxDrawCall, GfxDrawInstruction, Point, Rectangle, RenderApiPtr},
    prop::{PropertyBool, PropertyPtr, PropertyRect, PropertyUint32, Role},
    scene::{Pimpl, SceneGraph, SceneGraphPtr2, SceneNodeId},
    ExecutorPtr,
};

use super::{
    get_child_nodes_ordered, get_parent_rect, get_ui_object, DrawUpdate, OnModify, Stoppable,
    UIObject,
};

pub type LayerPtr = Arc<Layer>;

pub struct Layer {
    sg: SceneGraphPtr2,
    node_id: SceneNodeId,
    // Task is dropped at the end of the scope for Layer, hence ending it
    #[allow(dead_code)]
    tasks: Vec<smol::Task<()>>,
    render_api: RenderApiPtr,

    dc_key: u64,

    is_visible: PropertyBool,
    rect: PropertyRect,
    z_index: PropertyUint32,
}

impl Layer {
    pub async fn new(
        ex: ExecutorPtr,
        sg_ptr: SceneGraphPtr2,
        node_id: SceneNodeId,
        render_api: RenderApiPtr,
    ) -> Pimpl {
        let sg = sg_ptr.lock().await;
        let node = sg.get_node(node_id).unwrap();
        let node_name = node.name.clone();

        let is_visible =
            PropertyBool::wrap(node, Role::Internal, "is_visible", 0).expect("Layer::is_visible");
        let rect = PropertyRect::wrap(node, Role::Internal, "rect").unwrap();
        let z_index = PropertyUint32::wrap(node, Role::Internal, "z_index", 0).unwrap();
        drop(sg);

        let self_ = Arc::new_cyclic(|me: &Weak<Self>| {
            let mut on_modify = OnModify::new(ex.clone(), node_name, node_id, me.clone());
            on_modify.when_change(rect.prop(), Self::redraw);

            Self {
                sg: sg_ptr,
                node_id,
                tasks: on_modify.tasks,
                render_api,
                dc_key: OsRng.gen(),
                is_visible,
                rect,
                z_index,
            }
        });

        Pimpl::Layer(self_)
    }

    pub async fn handle_char(
        &self,
        sg: &SceneGraph,
        key: char,
        mods: KeyMods,
        repeat: bool,
    ) -> bool {
        false
    }

    async fn redraw(self: Arc<Self>) {
        let sg = self.sg.lock().await;
        let node = sg.get_node(self.node_id).unwrap();

        let Some(parent_rect) = get_parent_rect(&sg, node) else {
            return;
        };

        let Some(draw_update) = self.draw(&sg, &parent_rect).await else {
            error!(target: "ui::layer", "Layer {:?} failed to draw", node);
            return;
        };
        self.render_api.replace_draw_calls(draw_update.draw_calls);
        debug!(target: "ui::layer", "replace draw calls done");
    }
}

impl Stoppable for Layer {
    async fn stop(&self) {}
}

#[async_trait]
impl UIObject for Layer {
    fn z_index(&self) -> u32 {
        self.z_index.get()
    }

    async fn draw(&self, sg: &SceneGraph, parent_rect: &Rectangle) -> Option<DrawUpdate> {
        debug!(target: "ui::layer", "Layer::draw()");
        let node = sg.get_node(self.node_id).unwrap();

        if !self.is_visible.get() {
            debug!(target: "ui::layer", "invisible layer node '{}':{}", node.name, node.id);
            return None
        }

        self.rect.eval(parent_rect).ok()?;

        let mut screen_rect = self.rect.get() + parent_rect.pos();

        if !parent_rect.includes(&screen_rect) {
            error!(
                target: "ui::layer",
                "layer '{}':{} rect {:?} is not inside parent {:?}",
                node.name, node.id, screen_rect, parent_rect
            );
            return None
        }

        debug!(target: "ui::layer", "Parent rect: {:?}", parent_rect);
        debug!(target: "ui::layer", "Viewport rect: {:?}", screen_rect);

        // Apply viewport

        let mut draw_calls = vec![];
        let mut child_calls = vec![];
        let mut freed_textures = vec![];
        let mut freed_buffers = vec![];

        for child_id in get_child_nodes_ordered(&sg, self.node_id) {
            let node = sg.get_node(child_id).unwrap();
            let obj = get_ui_object(node);
            let Some(mut draw_update) = obj.draw(sg, &screen_rect).await else {
                debug!(target: "ui::layer", "Skipped draw() of {node:?}");
                continue
            };

            draw_calls.append(&mut draw_update.draw_calls);
            child_calls.push(draw_update.key);
            freed_textures.append(&mut draw_update.freed_textures);
            freed_buffers.append(&mut draw_update.freed_buffers);
        }

        let dc = GfxDrawCall {
            instrs: vec![GfxDrawInstruction::ApplyViewport(screen_rect)],
            dcs: child_calls,
            z_index: 0,
        };
        draw_calls.push((self.dc_key, dc));
        Some(DrawUpdate { key: self.dc_key, draw_calls, freed_textures, freed_buffers })
    }

    async fn handle_char(&self, sg: &SceneGraph, key: char, mods: KeyMods, repeat: bool) -> bool {
        for child_id in get_child_nodes_ordered(&sg, self.node_id) {
            let node = sg.get_node(child_id).unwrap();
            let obj = get_ui_object(node);
            if obj.handle_char(sg, key, mods, repeat).await {
                return true
            }
        }
        false
    }

    async fn handle_key_down(
        &self,
        sg: &SceneGraph,
        key: KeyCode,
        mods: KeyMods,
        repeat: bool,
    ) -> bool {
        for child_id in get_child_nodes_ordered(&sg, self.node_id) {
            let node = sg.get_node(child_id).unwrap();
            let obj = get_ui_object(node);
            if obj.handle_key_down(sg, key, mods, repeat).await {
                return true
            }
        }
        false
    }

    async fn handle_key_up(&self, sg: &SceneGraph, key: KeyCode, mods: KeyMods) -> bool {
        for child_id in get_child_nodes_ordered(&sg, self.node_id) {
            let node = sg.get_node(child_id).unwrap();
            let obj = get_ui_object(node);
            if obj.handle_key_up(sg, key, mods).await {
                return true
            }
        }
        false
    }
    async fn handle_mouse_btn_down(
        &self,
        sg: &SceneGraph,
        btn: MouseButton,
        mut mouse_pos: Point,
    ) -> bool {
        mouse_pos -= self.rect.get().pos();
        for child_id in get_child_nodes_ordered(&sg, self.node_id) {
            let node = sg.get_node(child_id).unwrap();
            let obj = get_ui_object(node);
            if obj.handle_mouse_btn_down(sg, btn, mouse_pos).await {
                return true
            }
        }
        false
    }
    async fn handle_mouse_btn_up(
        &self,
        sg: &SceneGraph,
        btn: MouseButton,
        mut mouse_pos: Point,
    ) -> bool {
        mouse_pos -= self.rect.get().pos();
        for child_id in get_child_nodes_ordered(&sg, self.node_id) {
            let node = sg.get_node(child_id).unwrap();
            let obj = get_ui_object(node);
            if obj.handle_mouse_btn_up(sg, btn, mouse_pos).await {
                return true
            }
        }
        false
    }
    async fn handle_mouse_move(&self, sg: &SceneGraph, mut mouse_pos: Point) -> bool {
        mouse_pos -= self.rect.get().pos();
        for child_id in get_child_nodes_ordered(&sg, self.node_id) {
            let node = sg.get_node(child_id).unwrap();
            let obj = get_ui_object(node);
            if obj.handle_mouse_move(sg, mouse_pos).await {
                return true
            }
        }
        false
    }
    async fn handle_mouse_wheel(&self, sg: &SceneGraph, mut wheel_pos: Point) -> bool {
        wheel_pos -= self.rect.get().pos();
        for child_id in get_child_nodes_ordered(&sg, self.node_id) {
            let node = sg.get_node(child_id).unwrap();
            let obj = get_ui_object(node);
            if obj.handle_mouse_wheel(sg, wheel_pos).await {
                return true
            }
        }
        false
    }
    async fn handle_touch(
        &self,
        sg: &SceneGraph,
        phase: TouchPhase,
        id: u64,
        mut touch_pos: Point,
    ) -> bool {
        touch_pos -= self.rect.get().pos();
        for child_id in get_child_nodes_ordered(&sg, self.node_id) {
            let node = sg.get_node(child_id).unwrap();
            let obj = get_ui_object(node);
            if obj.handle_touch(sg, phase, id, touch_pos).await {
                return true
            }
        }
        false
    }
}
