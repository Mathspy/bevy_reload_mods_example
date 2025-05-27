use bevy::{
    DefaultPlugins,
    app::{App, Startup, Update},
    color::Color,
    core_pipeline::core_2d::Camera2d,
    ecs::{
        children,
        query::{Changed, With},
        spawn::SpawnRelated,
        system::{Commands, Query},
    },
    ui::{
        AlignItems, BorderColor, Interaction, JustifyContent, Node, UiRect, Val,
        widget::{Button, Text},
    },
    utils::default,
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, button_clicked)
        .run();
}

fn button_clicked(
    mut interaction_query: Query<&Interaction, (Changed<Interaction>, With<Button>)>,
) {
    for interaction in &mut interaction_query {
        if *interaction == Interaction::Pressed {
            println!("Pressed!");
        }
    }
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        children![(
            Button,
            Node {
                padding: UiRect::axes(Val::Px(50.0), Val::Px(20.0)),
                border: UiRect::all(Val::Px(5.0)),
                // horizontally center child text
                justify_content: JustifyContent::Center,
                // vertically center child text
                align_items: AlignItems::Center,
                ..default()
            },
            BorderColor(Color::BLACK),
            children![(Text::new("Reload Mods"),)]
        )],
    ));
}
