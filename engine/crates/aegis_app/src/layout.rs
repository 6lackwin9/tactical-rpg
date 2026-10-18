use taffy::prelude::*;

pub struct AppLayout {
    pub taffy: TaffyTree<()>,
    pub root: NodeId,
    pub top_bar: NodeId,
    pub main_area: NodeId,
    pub left_panel: NodeId,
    pub center_arena: NodeId,
    pub right_panel: NodeId,
    pub bottom_panel: NodeId,
}

impl AppLayout {
    pub fn new() -> Self {
        let mut taffy: TaffyTree<()> = TaffyTree::new();

        let top_bar = taffy
            .new_leaf(Style {
                size: Size {
                    width: Dimension::Percent(1.0),
                    height: Dimension::Length(48.0),
                },
                margin: Rect {
                    left: LengthPercentageAuto::Length(8.0),
                    right: LengthPercentageAuto::Length(8.0),
                    top: LengthPercentageAuto::Length(8.0),
                    bottom: LengthPercentageAuto::Length(4.0),
                },
                ..Default::default()
            })
            .unwrap();

        let left_panel = taffy
            .new_leaf(Style {
                size: Size {
                    width: Dimension::Length(260.0),
                    height: Dimension::Percent(1.0),
                },
                margin: Rect {
                    left: LengthPercentageAuto::Length(8.0),
                    right: LengthPercentageAuto::Length(4.0),
                    top: LengthPercentageAuto::Length(4.0),
                    bottom: LengthPercentageAuto::Length(4.0),
                },
                ..Default::default()
            })
            .unwrap();

        let center_arena = taffy
            .new_leaf(Style {
                flex_grow: 1.0,
                size: Size {
                    width: Dimension::Auto,
                    height: Dimension::Percent(1.0),
                },
                margin: Rect {
                    left: LengthPercentageAuto::Length(4.0),
                    right: LengthPercentageAuto::Length(4.0),
                    top: LengthPercentageAuto::Length(4.0),
                    bottom: LengthPercentageAuto::Length(4.0),
                },
                ..Default::default()
            })
            .unwrap();

        let right_panel = taffy
            .new_leaf(Style {
                size: Size {
                    width: Dimension::Length(260.0),
                    height: Dimension::Percent(1.0),
                },
                margin: Rect {
                    left: LengthPercentageAuto::Length(4.0),
                    right: LengthPercentageAuto::Length(8.0),
                    top: LengthPercentageAuto::Length(4.0),
                    bottom: LengthPercentageAuto::Length(4.0),
                },
                ..Default::default()
            })
            .unwrap();

        let main_area = taffy
            .new_with_children(
                Style {
                    flex_direction: FlexDirection::Row,
                    flex_grow: 1.0,
                    size: Size {
                        width: Dimension::Percent(1.0),
                        height: Dimension::Auto,
                    },
                    ..Default::default()
                },
                &[left_panel, center_arena, right_panel],
            )
            .unwrap();

        let bottom_panel = taffy
            .new_leaf(Style {
                size: Size {
                    width: Dimension::Percent(1.0),
                    height: Dimension::Length(120.0),
                },
                margin: Rect {
                    left: LengthPercentageAuto::Length(8.0),
                    right: LengthPercentageAuto::Length(8.0),
                    top: LengthPercentageAuto::Length(4.0),
                    bottom: LengthPercentageAuto::Length(8.0),
                },
                ..Default::default()
            })
            .unwrap();

        let root = taffy
            .new_with_children(
                Style {
                    flex_direction: FlexDirection::Column,
                    size: Size {
                        width: Dimension::Percent(1.0),
                        height: Dimension::Percent(1.0),
                    },
                    ..Default::default()
                },
                &[top_bar, main_area, bottom_panel],
            )
            .unwrap();

        Self {
            taffy,
            root,
            top_bar,
            main_area,
            left_panel,
            center_arena,
            right_panel,
            bottom_panel,
        }
    }

    pub fn compute(&mut self, width: f32, height: f32) {
        let _ = self.taffy.compute_layout(
            self.root,
            Size {
                width: AvailableSpace::Definite(width),
                height: AvailableSpace::Definite(height),
            },
        );
    }
}
