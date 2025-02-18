/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

extern crate app_units;
extern crate euclid;
extern crate gleam;
extern crate glutin;
extern crate webrender;

#[path = "common/boilerplate.rs"]
mod boilerplate;

use app_units::Au;
use boilerplate::{Example, HandyDandyRectBuilder};
use euclid::vec2;
use glutin::TouchPhase;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use webrender::api::*;

#[derive(Debug)]
enum Gesture {
    None,
    Pan,
    Zoom,
}

#[derive(Debug)]
struct Touch {
    id: u64,
    start_x: f32,
    start_y: f32,
    current_x: f32,
    current_y: f32,
}

fn dist(x0: f32, y0: f32, x1: f32, y1: f32) -> f32 {
    let dx = x0 - x1;
    let dy = y0 - y1;
    ((dx * dx) + (dy * dy)).sqrt()
}

impl Touch {
    fn distance_from_start(&self) -> f32 {
        dist(self.start_x, self.start_y, self.current_x, self.current_y)
    }

    fn initial_distance_from_other(&self, other: &Touch) -> f32 {
        dist(self.start_x, self.start_y, other.start_x, other.start_y)
    }

    fn current_distance_from_other(&self, other: &Touch) -> f32 {
        dist(
            self.current_x,
            self.current_y,
            other.current_x,
            other.current_y,
        )
    }
}

struct TouchState {
    active_touches: HashMap<u64, Touch>,
    current_gesture: Gesture,
    start_zoom: f32,
    current_zoom: f32,
    start_pan: DeviceIntPoint,
    current_pan: DeviceIntPoint,
}

enum TouchResult {
    None,
    Pan(DeviceIntPoint),
    Zoom(f32),
}

impl TouchState {
    fn new() -> TouchState {
        TouchState {
            active_touches: HashMap::new(),
            current_gesture: Gesture::None,
            start_zoom: 1.0,
            current_zoom: 1.0,
            start_pan: DeviceIntPoint::zero(),
            current_pan: DeviceIntPoint::zero(),
        }
    }

    fn handle_event(&mut self, touch: glutin::Touch) -> TouchResult {
        match touch.phase {
            TouchPhase::Started => {
                debug_assert!(!self.active_touches.contains_key(&touch.id));
                self.active_touches.insert(
                    touch.id,
                    Touch {
                        id: touch.id,
                        start_x: touch.location.0 as f32,
                        start_y: touch.location.1 as f32,
                        current_x: touch.location.0 as f32,
                        current_y: touch.location.1 as f32,
                    },
                );
                self.current_gesture = Gesture::None;
            }
            TouchPhase::Moved => {
                match self.active_touches.get_mut(&touch.id) {
                    Some(active_touch) => {
                        active_touch.current_x = touch.location.0 as f32;
                        active_touch.current_y = touch.location.1 as f32;
                    }
                    None => panic!("move touch event with unknown touch id!"),
                }

                match self.current_gesture {
                    Gesture::None => {
                        let mut over_threshold_count = 0;
                        let active_touch_count = self.active_touches.len();

                        for (_, touch) in &self.active_touches {
                            if touch.distance_from_start() > 8.0 {
                                over_threshold_count += 1;
                            }
                        }

                        if active_touch_count == over_threshold_count {
                            if active_touch_count == 1 {
                                self.start_pan = self.current_pan;
                                self.current_gesture = Gesture::Pan;
                            } else if active_touch_count == 2 {
                                self.start_zoom = self.current_zoom;
                                self.current_gesture = Gesture::Zoom;
                            }
                        }
                    }
                    Gesture::Pan => {
                        let keys: Vec<u64> = self.active_touches.keys().cloned().collect();
                        debug_assert!(keys.len() == 1);
                        let active_touch = &self.active_touches[&keys[0]];
                        let x = active_touch.current_x - active_touch.start_x;
                        let y = active_touch.current_y - active_touch.start_y;
                        self.current_pan.x = self.start_pan.x + x.round() as i32;
                        self.current_pan.y = self.start_pan.y + y.round() as i32;
                        return TouchResult::Pan(self.current_pan);
                    }
                    Gesture::Zoom => {
                        let keys: Vec<u64> = self.active_touches.keys().cloned().collect();
                        debug_assert!(keys.len() == 2);
                        let touch0 = &self.active_touches[&keys[0]];
                        let touch1 = &self.active_touches[&keys[1]];
                        let initial_distance = touch0.initial_distance_from_other(touch1);
                        let current_distance = touch0.current_distance_from_other(touch1);
                        self.current_zoom = self.start_zoom * current_distance / initial_distance;
                        return TouchResult::Zoom(self.current_zoom);
                    }
                }
            }
            TouchPhase::Ended | TouchPhase::Cancelled => {
                self.active_touches.remove(&touch.id).unwrap();
                self.current_gesture = Gesture::None;
            }
        }

        TouchResult::None
    }
}

fn load_file(name: &str) -> Vec<u8> {
    let mut file = File::open(name).unwrap();
    let mut buffer = vec![];
    file.read_to_end(&mut buffer).unwrap();
    buffer
}

fn main() {
    let mut app = App {
        touch_state: TouchState::new(),
    };
    boilerplate::main_wrapper(&mut app, None);
}

struct App {
    touch_state: TouchState,
}

impl Example for App {
    fn render(
        &mut self,
        api: &RenderApi,
        builder: &mut DisplayListBuilder,
        resources: &mut ResourceUpdates,
        layout_size: LayoutSize,
        _pipeline_id: PipelineId,
        _document_id: DocumentId,
    ) {
        let bounds = LayoutRect::new(LayoutPoint::zero(), layout_size);
        let info = LayoutPrimitiveInfo::new(bounds);
        builder.push_stacking_context(
            &info,
            ScrollPolicy::Scrollable,
            None,
            TransformStyle::Flat,
            None,
            MixBlendMode::Normal,
            Vec::new(),
        );

        let image_mask_key = api.generate_image_key();
        resources.add_image(
            image_mask_key,
            ImageDescriptor::new(2, 2, ImageFormat::A8, true),
            ImageData::new(vec![0, 80, 180, 255]),
            None,
        );
        let mask = ImageMask {
            image: image_mask_key,
            rect: (75, 75).by(100, 100),
            repeat: false,
        };
        let complex = ComplexClipRegion::new((50, 50).to(150, 150), BorderRadius::uniform(20.0));
        let id = builder.define_clip(None, bounds, vec![complex], Some(mask));
        builder.push_clip_id(id);

        let info = LayoutPrimitiveInfo::new((100, 100).to(200, 200));
        builder.push_rect(&info, ColorF::new(0.0, 1.0, 0.0, 1.0));

        let info = LayoutPrimitiveInfo::new((250, 100).to(350, 200));
        builder.push_rect(&info, ColorF::new(0.0, 1.0, 0.0, 1.0));
        let border_side = BorderSide {
            color: ColorF::new(0.0, 0.0, 1.0, 1.0),
            style: BorderStyle::Groove,
        };
        let border_widths = BorderWidths {
            top: 10.0,
            left: 10.0,
            bottom: 10.0,
            right: 10.0,
        };
        let border_details = BorderDetails::Normal(NormalBorder {
            top: border_side,
            right: border_side,
            bottom: border_side,
            left: border_side,
            radius: BorderRadius::uniform(20.0),
        });

        let info = LayoutPrimitiveInfo::new((100, 100).to(200, 200));
        builder.push_border(&info, border_widths, border_details);


        if false {
            // draw text?
            let font_key = api.generate_font_key();
            let font_bytes = load_file("res/FreeSans.ttf");
            resources.add_raw_font(font_key, font_bytes, 0);

            let font_instance_key = api.generate_font_instance_key();
            resources.add_font_instance(font_instance_key, font_key, Au::from_px(32), None, None);

            let text_bounds = (100, 200).by(700, 300);
            let glyphs = vec![
                GlyphInstance {
                    index: 48,
                    point: LayoutPoint::new(100.0, 100.0),
                },
                GlyphInstance {
                    index: 68,
                    point: LayoutPoint::new(150.0, 100.0),
                },
                GlyphInstance {
                    index: 80,
                    point: LayoutPoint::new(200.0, 100.0),
                },
                GlyphInstance {
                    index: 82,
                    point: LayoutPoint::new(250.0, 100.0),
                },
                GlyphInstance {
                    index: 81,
                    point: LayoutPoint::new(300.0, 100.0),
                },
                GlyphInstance {
                    index: 3,
                    point: LayoutPoint::new(350.0, 100.0),
                },
                GlyphInstance {
                    index: 86,
                    point: LayoutPoint::new(400.0, 100.0),
                },
                GlyphInstance {
                    index: 79,
                    point: LayoutPoint::new(450.0, 100.0),
                },
                GlyphInstance {
                    index: 72,
                    point: LayoutPoint::new(500.0, 100.0),
                },
                GlyphInstance {
                    index: 83,
                    point: LayoutPoint::new(550.0, 100.0),
                },
                GlyphInstance {
                    index: 87,
                    point: LayoutPoint::new(600.0, 100.0),
                },
                GlyphInstance {
                    index: 17,
                    point: LayoutPoint::new(650.0, 100.0),
                },
            ];

            let info = LayoutPrimitiveInfo::new(text_bounds);
            builder.push_text(
                &info,
                &glyphs,
                font_instance_key,
                ColorF::new(1.0, 1.0, 0.0, 1.0),
                None,
            );
        }

        if false {
            // draw box shadow?
            let rect = LayoutRect::zero();
            let simple_box_bounds = (20, 200).by(50, 50);
            let offset = vec2(10.0, 10.0);
            let color = ColorF::new(1.0, 1.0, 1.0, 1.0);
            let blur_radius = 0.0;
            let spread_radius = 0.0;
            let simple_border_radius = 8.0;
            let box_shadow_type = BoxShadowClipMode::Inset;
            let info = LayoutPrimitiveInfo::with_clip_rect(rect, bounds);

            builder.push_box_shadow(
                &info,
                simple_box_bounds,
                offset,
                color,
                blur_radius,
                spread_radius,
                simple_border_radius,
                box_shadow_type,
            );
        }

        builder.pop_clip_id();
        builder.pop_stacking_context();
    }

    fn on_event(&mut self, event: glutin::Event, api: &RenderApi, document_id: DocumentId) -> bool {
        match event {
            glutin::Event::Touch(touch) => match self.touch_state.handle_event(touch) {
                TouchResult::Pan(pan) => {
                    api.set_pan(document_id, pan);
                    api.generate_frame(document_id, None);
                }
                TouchResult::Zoom(zoom) => {
                    api.set_pinch_zoom(document_id, ZoomFactor::new(zoom));
                    api.generate_frame(document_id, None);
                }
                TouchResult::None => {}
            },
            _ => (),
        }

        false
    }
}
