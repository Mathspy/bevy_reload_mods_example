#![allow(clippy::type_complexity)]

use bevy::{
    DefaultPlugins,
    app::{App, Startup, Update},
    color::Color,
    core_pipeline::core_2d::Camera2d,
    ecs::{
        children,
        component::Component,
        query::{Changed, With},
        spawn::SpawnRelated,
        system::{Commands, Query, Single},
    },
    ui::{
        AlignItems, BorderColor, Interaction, JustifyContent, Node, UiRect, Val,
        widget::{Button, Text},
    },
    utils::default,
};
use nom::IResult;

#[derive(Debug)]
enum ImagineStage {
    Update,
    Last,
}

#[derive(Debug)]
enum ImagineStatement {
    Print { text: String },
}

#[derive(Debug)]
struct ImagineFunction {
    stage: ImagineStage,
    name: String,
    body: Vec<ImagineStatement>,
}

#[derive(Debug)]
struct ImagineFile {
    items: Vec<ImagineFunction>,
}

impl ImagineFile {
    fn parse(input: &str) -> Result<ImagineFile, nom::Err<nom::error::Error<&str>>> {
        use nom::{
            Parser,
            branch::alt,
            bytes::complete::{is_not, tag},
            character::complete::{alpha1, alphanumeric1, multispace0, newline},
            combinator::{eof, map, recognize},
            error::ParseError,
            multi::{many0_count, separated_list1},
            sequence::{delimited, pair},
        };

        pub fn ws<'a, O, E: ParseError<&'a str>, F>(
            inner: F,
        ) -> impl Parser<&'a str, Output = O, Error = E>
        where
            F: Parser<&'a str, Output = O, Error = E>,
        {
            delimited(multispace0, inner, multispace0)
        }

        fn identifier(input: &str) -> IResult<&str, &str> {
            recognize(pair(
                alt((alpha1, tag("_"))),
                many0_count(alt((alphanumeric1, tag("_")))),
            ))
            .parse(input)
        }

        fn statement(input: &str) -> IResult<&str, ImagineStatement> {
            let (input, _) = ws(tag("print")).parse(input)?;
            let (input, _) = tag("\"")(input)?;
            let (input, text) = is_not("\"")(input)?;
            let (input, _) = tag("\"")(input)?;

            Ok((
                input,
                ImagineStatement::Print {
                    text: text.to_string(),
                },
            ))
        }

        fn function_stage(input: &str) -> IResult<&str, ImagineStage> {
            let stage_name = alt((
                map(tag("Update"), |_| ImagineStage::Update),
                map(tag("Last"), |_| ImagineStage::Last),
            ));

            ws(delimited(tag("#["), stage_name, tag("]"))).parse(input)
        }

        fn function(input: &str) -> IResult<&str, ImagineFunction> {
            let (input, stage) = function_stage(input)?;

            let (input, _) = tag("fn ")(input)?;
            let (input, name) = identifier(input)?;
            let (input, _) = tag("()")(input)?;
            let (input, _) = ws(tag("{")).parse(input)?;
            let (input, statements) = separated_list1(newline, statement).parse(input)?;
            let (input, _) = ws(tag("}")).parse(input)?;

            Ok((
                input,
                ImagineFunction {
                    stage,
                    name: name.to_string(),
                    body: statements,
                },
            ))
        }

        let (input, items) = separated_list1(multispace0, function).parse(input)?;
        let (_, _) = eof(input)?;

        Ok(ImagineFile { items })
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, button_clicked)
        .run();
}

#[derive(Component)]
struct ButtonText;

fn button_clicked(
    mut interaction_query: Query<&Interaction, (Changed<Interaction>, With<Button>)>,
    mut text: Single<&mut Text, With<ButtonText>>,
) {
    for interaction in &mut interaction_query {
        if *interaction == Interaction::Pressed {
            text.0 = "Reloading...".to_string();
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
            children![(Text::new("Reload Mods"), ButtonText)]
        )],
    ));
}
