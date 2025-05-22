use std::rc::Rc;

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    text::Line,
    widgets::{
        Block, Borders, List, ListItem, ListState, StatefulWidget, StatefulWidgetRef, WidgetRef,
    },
};
use serde::{Deserialize, Serialize};

use crate::config::{Config, Styles};

#[derive(Debug, Default)]
pub struct LeaguesState {
    pub focused: bool,
    pub list_state: ListState,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct League {
    pub name: String,
    pub region: String,
    pub id: String,
    pub selected: bool,
}

impl League {
    fn to_list_item(&self, styles: &Styles) -> ListItem {
        ListItem::new(format!("{}", self.name)).style(match self.selected {
            true => styles.highlight,
            false => styles.default,
        })
    }
}

#[derive(Debug)]
pub struct Leagues {
    pub leagues: Vec<League>,
    config: Rc<Config>,
}

impl Leagues {
    pub fn new(config: Rc<Config>) -> Self {
        Self {
            leagues: Vec::new(),
            config: config,
        }
    }

    pub fn select(&mut self, state: &ListState) -> Option<(bool, String)> {
        if let Some(i) = state.selected() {
            if let Some(league) = self.leagues.get_mut(i) {
                league.selected = !league.selected;
                if league.selected {
                    return Some((true, league.id.clone()));
                } else {
                    return Some((false, league.id.clone()));
                }
            }
        }
        return None;
    }

    pub fn select_name(&mut self, to_select: &str) -> Option<String> {
        if let Some(league) = self.leagues.iter_mut().find(|l| l.name == to_select) {
            league.selected = true;
            Some(league.id.to_string())
        } else {
            None
        }
    }

    pub fn get_selected_ids(&self) -> Vec<String> {
        self.leagues
            .iter()
            .filter(|l| l.selected)
            .map(|l| l.id.to_string())
            .collect()
    }
}

impl StatefulWidgetRef for &Leagues {
    type State = LeaguesState;

    fn render_ref(&self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let styles = &self.config.style;

        let inner_area = {
            if let (Some(block), Some(set)) = (styles.border, styles.border_set) {
                let border_style = if state.focused {
                    styles.highlight
                } else {
                    styles.default
                };
                let block = Block::new()
                    .borders(Borders::all())
                    .border_type(block)
                    .border_style(border_style);
                block.render_ref(area, buf);
                let mut inner = block.inner(area);
                if inner.height >= 2 {
                    let hor = set.horizontal;
                    let left = set.vertical_right;
                    let right = set.vertical_left;

                    let a: Rect = Rect {
                        x: area.left(),
                        y: area.top() + 2,
                        width: area.width,
                        height: 1 as u16,
                    };

                    let date_header = Line::from(format!(
                        "{}{}{}",
                        left,
                        hor.repeat(area.width.saturating_sub(2) as usize),
                        right
                    ))
                    .style(border_style);
                    date_header.render_ref(a, buf);

                    let title_area: Rect = Rect {
                        x: area.left() + 1,
                        y: area.top() + 1,
                        width: area.width.saturating_sub(2),
                        height: 1 as u16,
                    };

                    let title = Line::from("Leagues").centered().style(styles.highlight);
                    title.render_ref(title_area, buf);

                    inner.y += 2;
                    inner.height = inner.height.saturating_sub(2);
                } else {
                    inner.y += 1;
                    inner.height = inner.height.saturating_sub(1);
                }
                inner
            } else {
                area
            }
        };

        let items: Vec<ListItem> = self
            .leagues
            .iter()
            .map(|l| l.to_list_item(styles))
            .collect();
        let list = List::new(items).highlight_symbol("* ");

        list.render(inner_area, buf, &mut state.list_state);
    }
}
