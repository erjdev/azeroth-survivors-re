//! Developer-only view of the actor attachment, using the existing noncombat menu hero.
use super::Mask;
use bevy::prelude::*;

#[derive(Component)]
struct PreviewImage;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(PostUpdate, show.after(super::sync));
}

fn show(
    mut commands: Commands,
    mask: Res<Mask>,
    cameras: Query<&Camera, With<super::ActorMaskView>>,
    mut images: Query<(&mut ImageNode, &mut Visibility), With<PreviewImage>>,
) {
    let active = mask.preview
        && mask
            .camera
            .is_some_and(|entity| cameras.get(entity).is_ok_and(|camera| camera.is_active));
    if let Ok((mut image, mut visibility)) = images.single_mut() {
        image.image = mask.image.clone();
        *visibility = if active {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
        return;
    }
    if !active {
        return;
    }
    commands.spawn((
        PreviewImage,
        ImageNode::new(mask.image.clone()),
        Node {
            position_type: PositionType::Absolute,
            right: Val::Percent(2.0),
            bottom: Val::Percent(2.0),
            width: Val::Percent(68.0),
            height: Val::Percent(68.0),
            ..default()
        },
        BackgroundColor(Color::BLACK),
        GlobalZIndex(900),
    ));
}
