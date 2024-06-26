use std::error::Error;

use crate::snake::Direction;

use self::math::pos::pos;
use self::math::size::size;
use self::render::bitmap::Bitmap;
use self::render::color::{alphacomp, Color};
use image::{ImageFormat, ImageResult};
use math::size::Size;
use minifb::{Key, KeyRepeat, MouseButton, MouseMode, Scale, ScaleMode, Window, WindowOptions};
use owo_colors::OwoColorize;
use render::{DrawCommand, Renderer, Rotate, SpritesheetId};
use snake::{Banana, SnaekSheet, SnakeGame};
use ui::{
	Anchor, FlexDirection, Mouse, UiContext, WidgetDim, WidgetFlags, WidgetId, WidgetLayout, WidgetPadding,
	WidgetProps, WidgetSize, WidgetSprite,
};

mod math;
mod render;
mod snake;
mod ui;

const WIDTH: u16 = 97;
const HEIGHT: u16 = 124;

fn main() {
	eprintln!("{}", "Snaek!!".yellow());

	match game() {
		Ok(_) => eprintln!("{}", "See you next time :)".green()),
		Err(e) => {
			eprintln!("{}", "The game crashed! D:".red());
			eprintln!("-> {}", e);
		}
	}
}

const IMG_ASCII_CHARS: &[u8] = include_bytes!("../assets/ascii-chars.png");
const IMG_SNAEKSHEET: &[u8] = include_bytes!("../assets/snaeksheet.png");

/// Loads a PNG from memory into a raw ARGB8 bitmap.
fn load_png_from_memory(png: &[u8]) -> ImageResult<Bitmap> {
	let img = image::load_from_memory_with_format(png, ImageFormat::Png)?;

	let size = size(img.width() as u16, img.height() as u16);

	let buffer = (img.into_rgba8().pixels())
		.map(|pixel| {
			let [r, g, b, a] = pixel.0;
			u32::from_le_bytes([b, g, r, a])
		})
		.collect::<Vec<u32>>();

	Ok(Bitmap::from_buffer(buffer, size))
}

const VIEWPORT_SIZE: Size = size(WIDTH, HEIGHT);

const SNAEK_BLACK: Color = Color::from_hex(0xff181425);

fn game() -> Result<(), Box<dyn Error>> {
	let ascii_bitmap = load_png_from_memory(IMG_ASCII_CHARS)?;

	let mut renderer = Renderer::new(Bitmap::new(VIEWPORT_SIZE), ascii_bitmap);
	let mut ui = UiContext::new(VIEWPORT_SIZE);

	let snaek_sheet_id = renderer.register_spritesheet(load_png_from_memory(IMG_SNAEKSHEET)?);
	let snaek_sheet = snake::snaek_sheet();

	let options = WindowOptions {
		borderless: true,
		title: true,
		resize: false,
		scale: Scale::X4,
		scale_mode: ScaleMode::Stretch,
		..Default::default()
	};

	let mut window = Window::new("Snaek", WIDTH as usize, HEIGHT as usize, options)?;
	window.set_target_fps(60);

	let mut snake_game = SnakeGame::new(size(11, 11));
	let mut next_direction = snake_game.direction();

	let mut debug = false;
	let mut show_game_over = false;

	let mut draw_cmds = Vec::new();
	let mut mouse = Mouse::default();
	let mut unscaled_mouse_pos = None;

	let mut frame_count: u64 = 0;

	'game_loop: while window.is_open() {
		// input handling
		if window.is_key_down(Key::Escape) {
			break;
		}

		if let Some(next_pos) = window.get_mouse_pos(MouseMode::Discard) {
			mouse.x = next_pos.0;
			mouse.y = next_pos.1;
		}

		mouse.l_pressed = (window.get_mouse_down(MouseButton::Left), mouse.l_pressed.0);
		mouse.r_pressed = (window.get_mouse_down(MouseButton::Right), mouse.r_pressed.0);
		mouse.m_pressed = (window.get_mouse_down(MouseButton::Middle), mouse.m_pressed.0);

		// snake input
		if window.is_key_pressed(Key::Up, KeyRepeat::No) || window.is_key_pressed(Key::W, KeyRepeat::No) {
			next_direction = Direction::Up;
		} else if window.is_key_pressed(Key::Right, KeyRepeat::No) || window.is_key_pressed(Key::D, KeyRepeat::No) {
			next_direction = Direction::Right;
		} else if window.is_key_pressed(Key::Down, KeyRepeat::No) || window.is_key_pressed(Key::S, KeyRepeat::No) {
			next_direction = Direction::Down;
		} else if window.is_key_pressed(Key::Left, KeyRepeat::No) || window.is_key_pressed(Key::A, KeyRepeat::No) {
			next_direction = Direction::Left;
		}

		draw_cmds.clear();
		draw_cmds.push(DrawCommand::Clear);

		// UI
		let window_frame = ui.build_widget(
			WidgetProps::new(wk!())
				.with_flags(WidgetFlags::DRAW_BACKGROUND | WidgetFlags::DRAW_BORDER)
				.with_color(Color::from_hex(0xffc0cbdc))
				.with_border_color(Color::from_hex(0xff181425))
				.with_border_width(1)
				.with_acf(Some(alphacomp::dst))
				.with_size(WidgetSize::fill())
				.with_padding(WidgetPadding::all(1))
				.with_layout(WidgetLayout::flex(FlexDirection::Vertical, 0)),
		);
		{
			let navbar = ui.build_widget(
				WidgetProps::new(wk!())
					.with_flags(WidgetFlags::CAN_CLICK)
					.with_size(WidgetSize::new(WidgetDim::Fill, WidgetDim::Fixed(8)))
					.with_layout(WidgetLayout::flex(FlexDirection::Horizontal, 0)),
			);
			{
				let snaek_icon = ui.build_widget(
					WidgetProps::simple_sprite(wk!(), snaek_sheet_id, snaek_sheet.snaek_icon)
						.with_size(WidgetSize::fixed(8, 8))
						.with_draw_offset(pos(1, 1))
						.with_layout(WidgetLayout::flex(FlexDirection::Horizontal, 0)),
				);
				ui.add_child(navbar.id(), snaek_icon.id());

				let filler = ui.build_widget(
					WidgetProps::new(wk!())
						.with_size(WidgetSize::fill())
						.with_padding(WidgetPadding::hv(2, 1)),
				);
				{
					let title = ui.build_widget(
						WidgetProps::text(wk!(), renderer.text("Snaek"))
							.with_anchor_origin(Anchor::BOTTOM_LEFT, Anchor::BOTTOM_LEFT)
							.with_mask_and(Some(SNAEK_BLACK)),
					);
					ui.add_child(filler.id(), title.id());
				}
				ui.add_child(navbar.id(), filler.id());

				let btn_close = ui.btn_icon(
					WidgetProps::new(wk!()).with_size(WidgetSize::fixed(7, 7)),
					WidgetProps::simple_sprite(wk!(), snaek_sheet_id, snaek_sheet.icon_close)
						.with_mask_and(Some(SNAEK_BLACK)),
					Color::from_hex(0xffe43b44),
				);
				ui.add_child(navbar.id(), btn_close.id());

				if btn_close.clicked() {
					break 'game_loop;
				}
			}
			ui.add_child(window_frame.id(), navbar.id());

			if navbar.pressed() {
				let (cpx, cpy) = window.get_unscaled_mouse_pos(MouseMode::Pass).unwrap_or_default();
				let (mpx, mpy) = unscaled_mouse_pos.unwrap_or((cpx, cpy));

				let (wpx, wpy) = window.get_position();
				window.set_position(wpx + (cpx - mpx).round() as isize, wpy + (cpy - mpy).round() as isize);

				unscaled_mouse_pos = Some((mpx, mpy));
			} else {
				unscaled_mouse_pos = None;
			}

			let game_frame = ui.build_widget(
				WidgetProps::nine_slice_sprite(wk!(), snaek_sheet_id, snaek_sheet.box_embossed)
					.with_acf(Some(alphacomp::dst))
					.with_size(WidgetSize::fill())
					.with_padding(WidgetPadding::trbl(4, 5, 5, 5))
					.with_layout(WidgetLayout::flex(FlexDirection::Vertical, 2)),
			);
			{
				let display_frame = ui.build_widget(
					WidgetProps::new(wk!())
						.with_size(WidgetSize::new(WidgetDim::Fill, WidgetDim::Hug))
						.with_layout(WidgetLayout::flex(FlexDirection::Horizontal, 3)),
				);
				{
					let big_display = ui.big_3digits_display(
						wk!(),
						snake_game.bananas_eaten() as usize,
						snaek_sheet_id,
						snaek_sheet.box_num_display,
						snaek_sheet.bignum_placeholder,
						&snaek_sheet.bignums,
					);
					ui.add_child(display_frame.id(), big_display.id());

					let middle_frame = ui.build_widget(
						WidgetProps::new(wk!())
							.with_size(WidgetSize::hug())
							.with_layout(WidgetLayout::flex(FlexDirection::Vertical, 2)),
					);
					{
						let icon_restart = ui.build_widget(
							WidgetProps::simple_sprite(wk!(), snaek_sheet_id, snaek_sheet.icon_restart)
								.with_anchor_origin(Anchor::CENTER, Anchor::CENTER)
								.with_acf(Some(alphacomp::xor)),
						);
						let btn_restart = ui.btn_box(
							WidgetProps::new(wk!())
								.with_size(WidgetSize::hug())
								.with_padding(WidgetPadding::hv(3, 2)),
							WidgetSprite::NineSlice(snaek_sheet_id, snaek_sheet.box_embossed),
							WidgetSprite::NineSlice(snaek_sheet_id, snaek_sheet.box_carved),
							icon_restart.id(),
						);
						ui.add_child(middle_frame.id(), btn_restart.id());

						if btn_restart.clicked() {
							snake_game.restart();
							show_game_over = false;
							next_direction = snake_game.direction();
						}

						let icon_playpause = {
							let sprite = match debug {
								true => snaek_sheet.icon_play,
								false => snaek_sheet.icon_debug,
							};

							ui.build_widget(
								WidgetProps::simple_sprite(wk!(), snaek_sheet_id, sprite)
									.with_anchor_origin(Anchor::CENTER, Anchor::CENTER)
									.with_acf(Some(alphacomp::xor)),
							)
						};
						let btn_playdebug = ui.btn_box(
							WidgetProps::new(wk!())
								.with_size(WidgetSize::hug())
								.with_padding(WidgetPadding::hv(3, 2)),
							WidgetSprite::NineSlice(snaek_sheet_id, snaek_sheet.box_embossed),
							WidgetSprite::NineSlice(snaek_sheet_id, snaek_sheet.box_carved),
							icon_playpause.id(),
						);
						ui.add_child(middle_frame.id(), btn_playdebug.id());

						if btn_playdebug.clicked() {
							debug = !debug;
						}
					}
					ui.add_child(display_frame.id(), middle_frame.id());

					let right_frame = ui.build_widget(
						WidgetProps::new(wk!())
							.with_size(WidgetSize::fill())
							.with_layout(WidgetLayout::flex(FlexDirection::Vertical, 2)),
					);
					{
						let text_holder = ui.build_widget(WidgetProps::new(wk!()).with_size(WidgetSize::fill()));
						{
							let text = ui.build_widget(
								WidgetProps::text(wk!(), renderer.text("Speykious"))
									.with_anchor_origin(Anchor::BOTTOM_LEFT, Anchor::BOTTOM_LEFT)
									.with_mask_and(Some(SNAEK_BLACK)),
							);
							ui.add_child(text_holder.id(), text.id());
						}
						ui.add_child(right_frame.id(), text_holder.id());

						let time_display = ui.time_display(
							wk!(),
							snake_game.duration(),
							snaek_sheet_id,
							snaek_sheet.box_num_display,
							snaek_sheet.num_colon,
							&snaek_sheet.nums,
						);
						ui.add_child(right_frame.id(), time_display.id());
					}
					ui.add_child(display_frame.id(), right_frame.id());
				}
				ui.add_child(game_frame.id(), display_frame.id());

				let playfield = ui.build_widget(
					WidgetProps::nine_slice_sprite(wk!(), snaek_sheet_id, snaek_sheet.box_playfield)
						.with_size(WidgetSize::fill())
						.with_padding(WidgetPadding::all(4)),
				);
				{
					let snake_container = ui.build_widget(
						WidgetProps::new(wk!())
							.with_flags(WidgetFlags::DRAW_BACKGROUND)
							.with_color(Color::from_hex(0xff262b44)),
					);
					{
						draw_snake_game(
							&snake_game,
							&mut ui,
							&renderer,
							snake_container.id(),
							snaek_sheet_id,
							&snaek_sheet,
							debug,
							&mut show_game_over,
						);
					}
					ui.add_child(playfield.id(), snake_container.id());
				}
				ui.add_child(game_frame.id(), playfield.id());
			}
			ui.add_child(window_frame.id(), game_frame.id());
		}
		ui.solve_layout();
		ui.draw_widgets(&mut draw_cmds);
		ui.free_untouched_widgets();
		ui.react(&mouse);

		snake_game.update_duration();
		if frame_count % (60 / 3) == 0 {
			let was_dead = snake_game.is_dead();

			snake_game.change_direction(next_direction);
			snake_game.update();
			next_direction = snake_game.direction();

			if snake_game.is_dead() && !was_dead {
				show_game_over = true;
			}
		}

		renderer.draw(&draw_cmds);

		window
			.update_with_buffer(renderer.first_framebuffer().pixels(), WIDTH as usize, HEIGHT as usize)
			.unwrap();

		frame_count += 1;
	}

	Ok(())
}

#[allow(clippy::too_many_arguments)]
fn draw_snake_game(
	snake_game: &SnakeGame,
	ui: &mut UiContext,
	renderer: &Renderer,
	container_id: WidgetId,
	snaek_sheet_id: SpritesheetId,
	snaek_sheet: &SnaekSheet,
	debug: bool,
	show_game_over: &mut bool,
) {
	let playfield_size = snake_game.size();
	for y in 0..playfield_size.h as i16 {
		for x in 0..playfield_size.w as i16 {
			let slot_pos = pos(x, y);
			let slot = snake_game.slot_at(slot_pos);

			let (ikey_x, ikey_y) = (slot_pos.x as u64, slot_pos.y as u64);
			let mut holder_props = WidgetProps::new(wk!(ikey_x, ikey_y))
				.with_size(WidgetSize::fixed(7, 7))
				.with_pos(slot_pos * 7);

			if debug {
				holder_props = holder_props
					.with_flags(WidgetFlags::DRAW_BORDER)
					.with_border_color(Color::from_hex(0xff333333))
					.with_border_width(1)
					.with_acf(Some(alphacomp::add));
			}

			let sprite_holder = ui.build_widget(holder_props);
			{
				if let Some(banana) = slot.banana() {
					let banana_sprite = match banana {
						Banana::Yellow => snaek_sheet.banana_yellow,
						Banana::Red => snaek_sheet.banana_red,
						Banana::Cyan => snaek_sheet.banana_cyan,
					};

					let sprite = ui.build_widget(
						WidgetProps::simple_sprite(wk!(), snaek_sheet_id, banana_sprite)
							.with_anchor_origin(Anchor::CENTER, Anchor::CENTER),
					);
					ui.add_child(sprite_holder.id(), sprite.id());
				}

				let is_straight = slot.direction_next() == slot.direction_prev().opposite();

				let snake_sprite = match (slot.has_snake_head(), slot.has_snake_tail()) {
					(true, true) if is_straight => {
						let rotate = match slot.direction_next() {
							Direction::Up => Rotate::R270,
							Direction::Right => Rotate::R0,
							Direction::Down => Rotate::R90,
							Direction::Left => Rotate::R180,
						};
						Some((snaek_sheet.snake_straight, rotate))
					}
					(true, true) => {
						use Direction as D;
						let rotate = match (slot.direction_next(), slot.direction_prev()) {
							(D::Up, D::Right) | (D::Right, D::Up) => Rotate::R270,
							(D::Right, D::Down) | (D::Down, D::Right) => Rotate::R0,
							(D::Down, D::Left) | (D::Left, D::Down) => Rotate::R90,
							(D::Left, D::Up) | (D::Up, D::Left) => Rotate::R180,
							_ => Rotate::R0,
						};
						Some((snaek_sheet.snake_gay, rotate))
					}
					(true, false) => {
						let rotate = match slot.direction_prev() {
							Direction::Up => Rotate::R90,
							Direction::Right => Rotate::R180,
							Direction::Down => Rotate::R270,
							Direction::Left => Rotate::R0,
						};
						Some((snaek_sheet.snake_head, rotate))
					}
					(false, true) => {
						let rotate = match slot.direction_next() {
							Direction::Up => Rotate::R0,
							Direction::Right => Rotate::R90,
							Direction::Down => Rotate::R180,
							Direction::Left => Rotate::R270,
						};
						Some((snaek_sheet.snake_end, rotate))
					}
					(false, false) => None,
				};

				if let Some((snake_sprite, rotate)) = snake_sprite {
					let sprite = ui.build_widget(
						WidgetProps::simple_sprite(wk!(ikey_x, ikey_y), snaek_sheet_id, snake_sprite)
							.with_rotate(rotate)
							.with_anchor_origin(Anchor::CENTER, Anchor::CENTER),
					);
					ui.add_child(sprite_holder.id(), sprite.id());
				}

				// debug sprites
				if debug {
					// direction next
					let (anchor, w, h) = match slot.direction_next() {
						Direction::Up => (Anchor::TOP_CENTER, 1, 2),
						Direction::Right => (Anchor::CENTER_RIGHT, 2, 1),
						Direction::Down => (Anchor::BOTTOM_CENTER, 1, 2),
						Direction::Left => (Anchor::CENTER_LEFT, 2, 1),
					};

					let sprite = ui.build_widget(
						WidgetProps::new(wk!(ikey_x, ikey_y))
							.with_flags(WidgetFlags::DRAW_BACKGROUND)
							.with_color(Color::from_hex(0xff116611))
							.with_size(WidgetSize::fixed(w, h))
							.with_anchor_origin(anchor, anchor)
							.with_acf(Some(alphacomp::add)),
					);
					ui.add_child(sprite_holder.id(), sprite.id());

					// direction prev
					let (anchor, w, h) = match slot.direction_prev() {
						Direction::Up => (Anchor::TOP_CENTER, 1, 3),
						Direction::Right => (Anchor::CENTER_RIGHT, 3, 1),
						Direction::Down => (Anchor::BOTTOM_CENTER, 1, 3),
						Direction::Left => (Anchor::CENTER_LEFT, 3, 1),
					};

					let sprite = ui.build_widget(
						WidgetProps::new(wk!(ikey_x, ikey_y))
							.with_flags(WidgetFlags::DRAW_BACKGROUND)
							.with_color(Color::from_hex(0xff661111))
							.with_size(WidgetSize::fixed(w, h))
							.with_anchor_origin(anchor, anchor)
							.with_acf(Some(alphacomp::add)),
					);
					ui.add_child(sprite_holder.id(), sprite.id());
				}
			}
			ui.add_child(container_id, sprite_holder.id());
		}
	}

	if snake_game.ate_banana() {
		let head_pos = snake_game.snake_head();

		let (rotate, anchor) = match snake_game.slot_at(head_pos).direction_prev() {
			Direction::Up => (Rotate::R90, Anchor::TOP_CENTER),
			Direction::Right => (Rotate::R180, Anchor::CENTER_RIGHT),
			Direction::Down => (Rotate::R270, Anchor::BOTTOM_CENTER),
			Direction::Left => (Rotate::R0, Anchor::CENTER_LEFT),
		};

		let tongue_pos = head_pos + snake_game.direction().pos_offset();
		let tongue_holder = ui.build_widget(
			WidgetProps::new(wk!())
				.with_size(WidgetSize::fixed(7, 7))
				.with_pos(tongue_pos * 7),
		);
		{
			let tongue = ui.build_widget(
				WidgetProps::simple_sprite(wk!(), snaek_sheet_id, snaek_sheet.snake_tongue)
					.with_anchor_origin(anchor, anchor)
					.with_rotate(rotate),
			);
			ui.add_child(tongue_holder.id(), tongue.id());
		}
		ui.add_child(container_id, tongue_holder.id());
	}

	if *show_game_over {
		let game_over_overlay = ui.build_widget(
			WidgetProps::new(wk!())
				.with_flags(WidgetFlags::DRAW_BACKGROUND)
				.with_color(Color::from_hex(0x80ffffff & SNAEK_BLACK.to_u32()))
				.with_size(WidgetSize::fill()),
		);
		{
			let column = ui.build_widget(
				WidgetProps::new(wk!())
					.with_size(WidgetSize::hug())
					.with_anchor_origin(Anchor::CENTER, Anchor::CENTER)
					.with_layout(WidgetLayout::flex(FlexDirection::Vertical, 4)),
			);
			{
				let game_over_text = ui.build_widget(WidgetProps::text(wk!(), renderer.text("Game Over! :(")));
				ui.add_child(column.id(), game_over_text.id());

				let oh_text =
					ui.build_widget(WidgetProps::text(wk!(), renderer.text("Oh")).with_mask_and(Some(SNAEK_BLACK)));

				let oh_btn = ui.btn_box(
					WidgetProps::new(wk!())
						.with_size(WidgetSize::hug())
						.with_anchor_origin(Anchor::TOP_CENTER, Anchor::TOP_CENTER)
						.with_padding(WidgetPadding::hv(4, 2)),
					WidgetSprite::NineSlice(snaek_sheet_id, snaek_sheet.box_embossed),
					WidgetSprite::NineSlice(snaek_sheet_id, snaek_sheet.box_carved),
					oh_text.id(),
				);
				ui.add_child(column.id(), oh_btn.id());

				if oh_btn.clicked() {
					*show_game_over = false;
				}
			}
			ui.add_child(game_over_overlay.id(), column.id());
		}
		ui.add_child(container_id, game_over_overlay.id());
	}
}
