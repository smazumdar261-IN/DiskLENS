use crate::models::{FileCategory, FileNode};
use eframe::egui::{Color32, Pos2, Rect, Stroke};
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct TreemapItem {
    pub name: String,
    pub path: PathBuf,
    pub size: u64,
    pub is_dir: bool,
    pub category: FileCategory,
    pub rect: Rect,
    pub depth: usize,
    pub children_count: usize,
    pub file_count: u64,
    pub dir_count: u64,
}

pub struct TreemapLayout {
    pub items: Vec<TreemapItem>,
}

impl TreemapLayout {
    pub fn build(root: &FileNode, bounds: Rect, max_depth: usize) -> Self {
        let mut items = Vec::new();
        if root.size == 0 || bounds.width() <= 2.0 || bounds.height() <= 2.0 {
            return Self { items };
        }

        layout_node(root, bounds, 0, max_depth, &mut items);
        Self { items }
    }
}

fn layout_node(
    node: &FileNode,
    rect: Rect,
    current_depth: usize,
    max_depth: usize,
    out_items: &mut Vec<TreemapItem>,
) {
    if rect.width() < 1.0 || rect.height() < 1.0 {
        return;
    }

    if !node.is_dir || current_depth >= max_depth || node.children.is_empty() {
        out_items.push(TreemapItem {
            name: node.name.clone(),
            path: node.path.clone(),
            size: node.size,
            is_dir: node.is_dir,
            category: node.category,
            rect,
            depth: current_depth,
            children_count: node.children.len(),
            file_count: node.file_count,
            dir_count: node.dir_count,
        });
        return;
    }

    let valid_children: Vec<&FileNode> = node.children.iter().filter(|c| c.size > 0).collect();
    if valid_children.is_empty() {
        out_items.push(TreemapItem {
            name: node.name.clone(),
            path: node.path.clone(),
            size: node.size,
            is_dir: node.is_dir,
            category: node.category,
            rect,
            depth: current_depth,
            children_count: 0,
            file_count: node.file_count,
            dir_count: node.dir_count,
        });
        return;
    }

    let total_size: f64 = valid_children.iter().map(|c| c.size as f64).sum();
    if total_size <= 0.0 {
        return;
    }

    let inner_rect = if current_depth == 0 {
        rect
    } else {
        rect.shrink(1.5)
    };

    let total_area = (inner_rect.width() * inner_rect.height()) as f64;
    let mut weights: Vec<(&FileNode, f64)> = valid_children
        .into_iter()
        .map(|c| {
            let item_area = (c.size as f64 / total_size) * total_area;
            (c, item_area)
        })
        .collect();

    weights.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    squarify(
        &weights,
        Vec::new(),
        inner_rect,
        current_depth,
        max_depth,
        out_items,
    );
}

fn worst_aspect_ratio(row: &[f64], width: f64) -> f64 {
    if row.is_empty() || width <= 0.0 {
        return f64::MAX;
    }
    let row_sum: f64 = row.iter().sum();
    if row_sum <= 0.0 {
        return f64::MAX;
    }
    let max_val = row.iter().copied().fold(f64::NAN, f64::max);
    let min_val = row.iter().copied().fold(f64::NAN, f64::min);
    let width_sq = width * width;
    let sum_sq = row_sum * row_sum;

    let ratio1 = (width_sq * max_val) / sum_sq;
    let ratio2 = sum_sq / (width_sq * min_val);
    ratio1.max(ratio2)
}

fn squarify<'a>(
    children: &[(&'a FileNode, f64)],
    mut current_row: Vec<(&'a FileNode, f64)>,
    bounds: Rect,
    current_depth: usize,
    max_depth: usize,
    out_items: &mut Vec<TreemapItem>,
) {
    if bounds.width() < 1.0 || bounds.height() < 1.0 {
        return;
    }

    if children.is_empty() {
        if !current_row.is_empty() {
            layout_row(&current_row, bounds, current_depth, max_depth, out_items);
        }
        return;
    }

    let shortest_side = (bounds.width().min(bounds.height())) as f64;
    let next_child = children[0];
    let remaining_children = &children[1..];

    if current_row.is_empty() {
        current_row.push(next_child);
        squarify(
            remaining_children,
            current_row,
            bounds,
            current_depth,
            max_depth,
            out_items,
        );
        return;
    }

    let row_areas: Vec<f64> = current_row.iter().map(|(_, a)| *a).collect();
    let current_worst = worst_aspect_ratio(&row_areas, shortest_side);

    let mut candidate_areas = row_areas.clone();
    candidate_areas.push(next_child.1);
    let candidate_worst = worst_aspect_ratio(&candidate_areas, shortest_side);

    if candidate_worst <= current_worst {
        current_row.push(next_child);
        squarify(
            remaining_children,
            current_row,
            bounds,
            current_depth,
            max_depth,
            out_items,
        );
    } else {
        let remaining_bounds =
            layout_row(&current_row, bounds, current_depth, max_depth, out_items);
        let mut new_row = Vec::new();
        new_row.push(next_child);
        squarify(
            remaining_children,
            new_row,
            remaining_bounds,
            current_depth,
            max_depth,
            out_items,
        );
    }
}

fn layout_row(
    row: &[(&FileNode, f64)],
    bounds: Rect,
    current_depth: usize,
    max_depth: usize,
    out_items: &mut Vec<TreemapItem>,
) -> Rect {
    let row_sum: f64 = row.iter().map(|(_, a)| *a).sum();
    let is_horizontal = bounds.width() >= bounds.height();

    if is_horizontal {
        let row_width = (row_sum / bounds.height() as f64) as f32;
        let clamped_row_width = row_width.min(bounds.width());

        let mut current_y = bounds.min.y;
        for (node, area) in row {
            let item_height = if row_sum > 0.0 {
                ((*area / row_sum) * bounds.height() as f64) as f32
            } else {
                0.0_f32
            };

            let item_rect = Rect::from_min_max(
                Pos2::new(bounds.min.x, current_y),
                Pos2::new(
                    bounds.min.x + clamped_row_width,
                    (current_y + item_height).min(bounds.max.y),
                ),
            );

            layout_node(node, item_rect, current_depth + 1, max_depth, out_items);
            current_y += item_height;
        }

        Rect::from_min_max(
            Pos2::new(bounds.min.x + clamped_row_width, bounds.min.y),
            bounds.max,
        )
    } else {
        let row_height = (row_sum / bounds.width() as f64) as f32;
        let clamped_row_height = row_height.min(bounds.height());

        let mut current_x = bounds.min.x;
        for (node, area) in row {
            let item_width = if row_sum > 0.0 {
                ((*area / row_sum) * bounds.width() as f64) as f32
            } else {
                0.0_f32
            };

            let item_rect = Rect::from_min_max(
                Pos2::new(current_x, bounds.min.y),
                Pos2::new(
                    (current_x + item_width).min(bounds.max.x),
                    bounds.min.y + clamped_row_height,
                ),
            );

            layout_node(node, item_rect, current_depth + 1, max_depth, out_items);
            current_x += item_width;
        }

        Rect::from_min_max(
            Pos2::new(bounds.min.x, bounds.min.y + clamped_row_height),
            bounds.max,
        )
    }
}

pub fn get_node_color(item: &TreemapItem, is_hovered: bool) -> (Color32, Stroke) {
    let mut base_color = if item.is_dir {
        match item.depth % 6 {
            0 => Color32::from_rgb(30, 41, 59),  // Slate Dark
            1 => Color32::from_rgb(30, 58, 138), // Indigo Navy
            2 => Color32::from_rgb(19, 78, 74),  // Teal Dark
            3 => Color32::from_rgb(76, 29, 149), // Violet Deep
            4 => Color32::from_rgb(113, 63, 18), // Amber Deep
            _ => Color32::from_rgb(55, 65, 81),  // Cool Gray
        }
    } else {
        item.category.color()
    };

    if is_hovered {
        base_color = Color32::from_rgb(
            base_color.r().saturating_add(55),
            base_color.g().saturating_add(55),
            base_color.b().saturating_add(55),
        );
        let stroke = Stroke::new(2.5_f32, Color32::from_rgb(0, 220, 255));
        (base_color, stroke)
    } else {
        let stroke = Stroke::new(1.0_f32, Color32::from_black_alpha(140));
        (base_color, stroke)
    }
}
