// OpenAOE: An open source reimplementation of Age of Empires (1997)
// Copyright (c) 2016 Kevin Fuller
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

extern crate open_aoe_drs as drs;
extern crate open_aoe_slp as slp;
extern crate open_aoe_palette as palette;
extern crate open_aoe_dat as dat;
extern crate open_aoe_language as language;
extern crate open_aoe_scn as scn;
extern crate open_aoe_media as media;
extern crate open_aoe_resource as resource;
extern crate open_aoe_types as types;
extern crate open_aoe_identifier as identifier;

#[macro_use]
extern crate lazy_static;

extern crate clap;

mod terrain;

use media::Key;
use terrain::TerrainBlender;
use terrain::TerrainRenderer;
use types::{Point, Rect};

use std::process;

// Doesn't currently match the camera speed in the original game
const CAMERA_SPEED: i32 = 1;

fn main() {
    let arg_matches = clap::App::new("OpenAOE")
        .about("An open source reimplementation of Age of Empires (1997)")
        .arg(clap::Arg::with_name("game_data_dir")
            .short("d")
            .long("game-data-dir")
            .value_name("GAME_DATA_DIR")
            .help("Sets the directory to look in for game data. Defaults to \"game\".")
            .takes_value(true))
        .arg(clap::Arg::with_name("SCENARIO")
            .required(true)
            .help("Scenario file to load (temporary while there's no menu)"))
        .get_matches();

    let game_data_dir = arg_matches.value_of("game_data_dir").unwrap_or("game");
    let scenario_file_name = arg_matches.value_of("SCENARIO").unwrap();

    let game_dir = match resource::GameDir::new(game_data_dir) {
        Ok(game_dir) => game_dir,
        Err(err) => {
            println!("{}", err);
            process::exit(1);
        }
    };

    let drs_manager = resource::DrsManager::new(&game_dir);
    if let Err(err) = drs_manager.borrow_mut().preload() {
        println!("Failed to preload DRS archives: {}", err);
        process::exit(1);
    }

    let shape_manager = match resource::ShapeManager::new(drs_manager.clone()) {
        Ok(shape_manager) => shape_manager,
        Err(err) => {
            println!("Failed to initialize the shape manager: {}", err);
            process::exit(1);
        }
    };

    println!("Loading \"data/empires.dat\"...");
    let empires = dat::EmpiresDb::read_from_file(game_dir.find_file("data/empires.dat").unwrap())
        .expect("data/empires.dat");

    println!("Loading \"{}\"...", scenario_file_name);
    let test_scn = scn::Scenario::read_from_file(scenario_file_name).expect(scenario_file_name);

    let mut media = match media::create_media(1024, 768, "OpenAOE") {
        Ok(media) => media,
        Err(err) => {
            println!("Failed to create media window: {}", err);
            process::exit(1);
        }
    };

    let tile_half_width = empires.terrain_block.tile_half_width as i32;
    let tile_half_height = empires.terrain_block.tile_half_height as i32;

    let mut terrain_renderer = TerrainRenderer::new(&empires.terrain_block);
    let terrain_blender = TerrainBlender::new(&empires.terrain_block,
                                              &test_scn.map.tiles,
                                              test_scn.map.width as isize,
                                              test_scn.map.height as isize);

    // Temporary hardcoded camera offset
    // let camera_pos = Point::new(0, -300);
    let mut camera_pos = Point::new(126 * tile_half_width, -145 * tile_half_height);
    // let camera_pos = Point::new(256 * tile_half_width, -300);
    // let camera_pos = Point::new(126 * tile_half_width, 125 * tile_half_height);

    while media.is_open() {
        media.update();

        media.renderer().present();

        media.renderer().set_camera_position(&camera_pos);

        // TODO: Render only the visible portion of the map
        let map_rect = Rect::of(0,
                                0,
                                terrain_blender.width() as i32,
                                terrain_blender.height() as i32);
        terrain_renderer.render(media.renderer(),
                                &mut *shape_manager.borrow_mut(),
                                &terrain_blender,
                                map_rect);

        if media.is_key_down(Key::Up) {
            camera_pos.y -= CAMERA_SPEED;
        } else if media.is_key_down(Key::Down) {
            camera_pos.y += CAMERA_SPEED;
        }
        if media.is_key_down(Key::Left) {
            camera_pos.x -= CAMERA_SPEED;
        } else if media.is_key_down(Key::Right) {
            camera_pos.x += CAMERA_SPEED;
        }
    }
}
