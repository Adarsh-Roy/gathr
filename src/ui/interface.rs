use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
    Frame,
};

use crate::ui::app::{App, AppMode};
use crate::fuzzy::filter::get_node_display_path;
use crate::directory::state::SelectionState;

pub fn draw_ui(f: &mut Frame, app: &mut App) {
    let size = f.size();
    app.viewport_height = size.height.saturating_sub(4) as usize; // Account for borders and status

    match app.mode {
        AppMode::Main => draw_main_interface(f, app, size),
        AppMode::Help => draw_help_interface(f, app, size),
    }
}

fn draw_main_interface(f: &mut Frame, app: &App, area: Rect) {
    // Clear the background for transparency
    f.render_widget(Clear, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Search bar
            Constraint::Min(0),    // File list
            Constraint::Length(3), // Status bar
        ])
        .split(area);

    draw_search_bar(f, app, chunks[0]);
    draw_file_list(f, app, chunks[1]);
    draw_status_bar(f, app, chunks[2]);
}

fn draw_search_bar(f: &mut Frame, app: &App, area: Rect) {
    let search_text = if app.search_query.is_empty() {
        "Type to search files and directories..."
    } else {
        &app.search_query
    };

    let style = if app.search_query.is_empty() {
        app.color_scheme.help_text
    } else {
        app.color_scheme.text
    };

    let search_paragraph = Paragraph::new(search_text)
        .style(style)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("🔍 Search")
                .border_style(app.color_scheme.border),
        )
        .wrap(Wrap { trim: true });

    f.render_widget(search_paragraph, area);
}

fn draw_file_list(f: &mut Frame, app: &App, area: Rect) {
    let items: Vec<ListItem> = app
        .filtered_results
        .visible_items
        .iter()
        .enumerate()
        .skip(app.scroll_offset)
        .take(area.height.saturating_sub(2) as usize)
        .map(|(display_index, &tree_index)| {
            create_list_item(app, tree_index, display_index + app.scroll_offset == app.selected_index)
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("📁 Files and Directories (► = cursor, Enter = toggle ✓/✗)")
                .border_style(app.color_scheme.border),
        )
        .style(app.color_scheme.background);

    f.render_widget(list, area);
}

fn create_list_item(app: &App, tree_index: usize, is_selected: bool) -> ListItem {
    if let Some(node) = app.tree.get_node(tree_index) {
        let display_path = get_node_display_path(&app.tree, tree_index);

        let state_indicator = match node.state {
            SelectionState::Included => "✓",
            SelectionState::Excluded => "✗",
            SelectionState::Partial => "◐",
        };

        let file_type_indicator = if node.is_directory {
            "📁"
        } else {
            "📄"
        };

        let cursor_indicator = if is_selected { "► " } else { "  " };

        // Get base style for the state, not influenced by selection
        let base_style = app.color_scheme.get_state_style(node.state);

        // Only the cursor indicator gets the selected style
        let cursor_style = if is_selected {
            app.color_scheme.selected
        } else {
            base_style
        };

        let spans = vec![
            Span::styled(cursor_indicator, cursor_style),
            Span::styled(format!("{} ", state_indicator), base_style),
            Span::styled(format!("{} ", file_type_indicator), app.color_scheme.text),
            Span::styled(display_path, base_style),
        ];

        if let Some(size) = node.size {
            let size_str = format_file_size(size);
            let line = Line::from(spans);

            // Add size information for files
            let mut full_spans = line.spans;
            full_spans.push(Span::styled(
                format!(" ({})", size_str),
                app.color_scheme.help_text,
            ));

            ListItem::new(Line::from(full_spans))
        } else {
            ListItem::new(Line::from(spans))
        }
    } else {
        ListItem::new("Invalid node")
    }
}

fn draw_status_bar(f: &mut Frame, app: &App, area: Rect) {
    let stats = app.get_stats();

    let left_text = format!(
        "Files: {}/{} | Size: {} | Filtered: {}",
        stats.included_files,
        stats.total_files,
        stats.format_size(),
        stats.filtered_count
    );

    // Adjust help text based on available width
    let available_width = area.width.saturating_sub(4) as usize; // Account for borders
    let left_text_len = left_text.len();
    let remaining_width = available_width.saturating_sub(left_text_len);

    let right_text = if remaining_width > 80 {
        "↑/↓: Move | Enter: Toggle ✓/✗ | Ctrl+E: Export | Ctrl+H: Help"
    } else if remaining_width > 60 {
        "↑/↓: Move | Enter: Toggle | Ctrl+E: Export | Ctrl+H: Help"
    } else if remaining_width > 40 {
        "↑/↓: Move | Ctrl+E: Export | Ctrl+H: Help"
    } else if remaining_width > 25 {
        "↑/↓: Move | Ctrl+E: Export"
    } else {
        "Ctrl+E: Export"
    };

    let status_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    let left_paragraph = Paragraph::new(left_text)
        .style(app.color_scheme.text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(app.color_scheme.border),
        );

    let right_paragraph = Paragraph::new(right_text)
        .style(app.color_scheme.help_text)
        .alignment(Alignment::Right)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(app.color_scheme.border),
        );

    f.render_widget(left_paragraph, status_chunks[0]);
    f.render_widget(right_paragraph, status_chunks[1]);
}

fn draw_help_interface(f: &mut Frame, app: &App, area: Rect) {
    let help_text = vec![
        Line::from("Text Ingest CLI - Help"),
        Line::from(""),
        Line::from("Search:"),
        Line::from("  Type       Add any character to search (letters, numbers, symbols)"),
        Line::from("  Backspace  Delete search character"),
        Line::from("  Esc        Clear search text (or quit if empty)"),
        Line::from(""),
        Line::from("Navigation:"),
        Line::from("  ↑/↓        Move up/down"),
        Line::from("  ←/→        Move up/down (alternative)"),
        Line::from("  Page Up    Page up"),
        Line::from("  Page Down  Page down"),
        Line::from("  Home       Go to top"),
        Line::from("  End        Go to bottom"),
        Line::from(""),
        Line::from("Selection:"),
        Line::from("  Enter      Toggle ✓ included / ✗ excluded"),
        Line::from(""),
        Line::from("Actions:"),
        Line::from("  Ctrl+E     Export output and quit"),
        Line::from("  Ctrl+H     Show this help"),
        Line::from("  Esc        Clear search (or quit if search empty)"),
        Line::from(""),
        Line::from("Colors:"),
        Line::from(vec![
            Span::styled("  ✓ ", app.color_scheme.included),
            Span::from("Included"),
        ]),
        Line::from(vec![
            Span::styled("  ✗ ", app.color_scheme.excluded),
            Span::from("Excluded"),
        ]),
        Line::from(vec![
            Span::styled("  ◐ ", app.color_scheme.partial),
            Span::from("Partially included"),
        ]),
        Line::from(""),
        Line::from("Press any key to return..."),
    ];

    let help_paragraph = Paragraph::new(help_text)
        .style(app.color_scheme.text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Help")
                .border_style(app.color_scheme.border),
        )
        .wrap(Wrap { trim: true });

    // Center the help dialog
    let popup_area = centered_rect(80, 90, area);
    f.render_widget(Clear, popup_area);
    f.render_widget(help_paragraph, popup_area);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

fn format_file_size(size: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size_f = size as f64;
    let mut unit_index = 0;

    while size_f >= 1024.0 && unit_index < UNITS.len() - 1 {
        size_f /= 1024.0;
        unit_index += 1;
    }

    if unit_index == 0 {
        format!("{} {}", size, UNITS[unit_index])
    } else {
        format!("{:.1} {}", size_f, UNITS[unit_index])
    }
}
