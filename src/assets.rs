use bevy::prelude::*;

#[derive(Resource, Clone)]
pub struct GameAssets {
    // UI & Board Sprites
    pub button_new_game: Handle<Image>,
    pub button_new_game_clicked: Handle<Image>,
    pub logo_win: Handle<Image>,
    pub background_top_left: Handle<Image>,
    pub background_side_left: Handle<Image>,

    pub board_corner_tl: Handle<Image>,
    pub board_corner_tr: Handle<Image>,
    pub board_corner_bl: Handle<Image>,
    pub board_corner_br: Handle<Image>,
    pub board_side_top: Handle<Image>,
    pub board_side_bottom: Handle<Image>,
    pub board_side_left: Handle<Image>,
    pub board_side_right: Handle<Image>,

    // Tile Sprites
    pub balloon: Handle<Image>,
    pub balloon_flagged: Handle<Image>,
    pub balloon_safe: Handle<Image>,
    pub crater_bomb: Handle<Image>,
    pub balloon_bits: [Handle<Image>; 4],
    pub numbers: [Handle<Image>; 9], // 0 is empty center, 1..8 are numbers

    // Animations (Horizontal Spritesheets & Layouts)
    pub pop_sheet: Handle<Image>,
    pub pop_layout: Handle<TextureAtlasLayout>,
    pub explode_sheet: Handle<Image>,
    pub explode_layout: Handle<TextureAtlasLayout>,
    pub dart_sheet: Handle<Image>,
    pub dart_layout: Handle<TextureAtlasLayout>,
    pub shadow_sheet: Handle<Image>,
    pub shadow_layout: Handle<TextureAtlasLayout>,

    // Audio SFX
    pub sfx_pops: Vec<Handle<AudioSource>>,
    pub sfx_bombs: Vec<Handle<AudioSource>>,
    pub sfx_click: Handle<AudioSource>,
    pub sfx_flag: Handle<AudioSource>,
    pub sfx_unflag: Handle<AudioSource>,
    pub sfx_dart: Handle<AudioSource>,
    pub sfx_win: Handle<AudioSource>,

    // Fonts
    pub font_slackey: Handle<Font>,
}

pub fn load_game_assets(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    // Animation Layouts
    // balloon_pop: 28 frames of 256x256 in 8 cols x 4 rows
    let pop_layout = texture_atlas_layouts.add(TextureAtlasLayout::from_grid(
        UVec2::splat(256),
        8,
        4,
        None,
        None,
    ));

    // balloon_explode: 24 frames of 256x256 in 8 cols x 3 rows
    let explode_layout = texture_atlas_layouts.add(TextureAtlasLayout::from_grid(
        UVec2::splat(256),
        8,
        3,
        None,
        None,
    ));

    // dart: 55 frames of 1024x768 in 8 cols x 7 rows
    let dart_layout = texture_atlas_layouts.add(TextureAtlasLayout::from_grid(
        UVec2::new(1024, 768),
        8,
        7,
        None,
        None,
    ));

    // shadow: 55 frames of 1024x768 in 8 cols x 7 rows
    let shadow_layout = texture_atlas_layouts.add(TextureAtlasLayout::from_grid(
        UVec2::new(1024, 768),
        8,
        7,
        None,
        None,
    ));

    // Audio SFX handles
    let mut sfx_pops = Vec::with_capacity(9);
    for i in 0..9 {
        sfx_pops.push(asset_server.load(format!("audio/pop{}.ogg", i)));
    }

    let mut sfx_bombs = Vec::with_capacity(5);
    for i in 0..5 {
        sfx_bombs.push(asset_server.load(format!("audio/bomb{}.ogg", i)));
    }

    let assets = GameAssets {
        button_new_game: asset_server.load("images/sprites/button_new_game.png"),
        button_new_game_clicked: asset_server.load("images/sprites/button_new_game_clicked.png"),
        logo_win: asset_server.load("images/sprites/logo_win.png"),
        background_top_left: asset_server.load("images/sprites/background_top_left.png"),
        background_side_left: asset_server.load("images/sprites/background_side_left.png"),

        board_corner_tl: asset_server.load("images/sprites/game_board_corner_top_left.png"),
        board_corner_tr: asset_server.load("images/sprites/game_board_corner_top_right.png"),
        board_corner_bl: asset_server.load("images/sprites/game_board_corner_bottom_left.png"),
        board_corner_br: asset_server.load("images/sprites/game_board_corner_bottom_right.png"),
        board_side_top: asset_server.load("images/sprites/game_board_side_top.png"),
        board_side_bottom: asset_server.load("images/sprites/game_board_side_bottom.png"),
        board_side_left: asset_server.load("images/sprites/game_board_side_left.png"),
        board_side_right: asset_server.load("images/sprites/game_board_side_right.png"),

        balloon: asset_server.load("images/sprites/balloon.png"),
        balloon_flagged: asset_server.load("images/sprites/balloon_tagged_frozen.png"),
        balloon_safe: asset_server.load("images/sprites/balloon_tagged_bomb.png"),
        crater_bomb: asset_server.load("images/sprites/crater_b.png"),
        balloon_bits: [
            asset_server.load("images/sprites/balloon_pieces_a.png"),
            asset_server.load("images/sprites/balloon_pieces_b.png"),
            asset_server.load("images/sprites/balloon_pieces_c.png"),
            asset_server.load("images/sprites/balloon_pieces_d.png"),
        ],
        numbers: [
            asset_server.load("images/sprites/game_board_center.png"),
            asset_server.load("images/sprites/number_one.png"),
            asset_server.load("images/sprites/number_two.png"),
            asset_server.load("images/sprites/number_three.png"),
            asset_server.load("images/sprites/number_four.png"),
            asset_server.load("images/sprites/number_five.png"),
            asset_server.load("images/sprites/number_six.png"),
            asset_server.load("images/sprites/number_seven.png"),
            asset_server.load("images/sprites/number_eight.png"),
        ],

        pop_sheet: asset_server.load("images/animations/balloon_pop_sheet.png"),
        pop_layout,
        explode_sheet: asset_server.load("images/animations/balloon_explode_sheet.png"),
        explode_layout,
        dart_sheet: asset_server.load("images/animations/dart_sheet.png"),
        dart_layout,
        shadow_sheet: asset_server.load("images/animations/shadow_sheet.png"),
        shadow_layout,

        sfx_pops,
        sfx_bombs,
        sfx_click: asset_server.load("audio/click.ogg"),
        sfx_flag: asset_server.load("audio/flag.ogg"),
        sfx_unflag: asset_server.load("audio/unflag.ogg"),
        sfx_dart: asset_server.load("audio/throwDart.ogg"),
        sfx_win: asset_server.load("audio/win.ogg"),

        font_slackey: asset_server.load("fonts/Slackey-Regular.ttf"),
    };

    commands.insert_resource(assets);
}
