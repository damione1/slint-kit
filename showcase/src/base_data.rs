// SPDX-License-Identifier: GPL-3.0-or-later
use super::ui;
use slint::{ComponentHandle, Model, ModelRc, VecModel};
use std::{cell::RefCell, collections::BTreeMap, rc::Rc};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ProductId(u32);
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Column {
    Title,
    Category,
    Price,
    Stock,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Order {
    Original,
    Ascending,
    Descending,
}
struct Product {
    id: ProductId,
    title: String,
    category: &'static str,
    price: kit::Money,
    stock: u32,
}
struct Catalogue {
    products: Vec<Product>,
    visible: Vec<usize>,
    selected: Option<ProductId>,
    column: Column,
    order: Order,
    generation: i32,
    rows: Rc<VecModel<ui::TableRow>>,
    articles: Rc<VecModel<ui::ContentItem>>,
    article: Option<usize>,
    lines: Rc<VecModel<ui::TextLine>>,
    line: Option<usize>,
    drafts: BTreeMap<usize, String>,
    line_generation: i32,
}
fn model<T: Clone + 'static>(data: Rc<VecModel<T>>) -> ModelRc<T> {
    ModelRc::from(data)
}
fn number(token: i32, len: usize) -> Option<usize> {
    usize::try_from(token)
        .ok()
        .and_then(|t| t.checked_sub(1))
        .filter(|i| *i < len)
}
fn navigate(
    index: Option<usize>,
    len: usize,
    step: ui::NavigationStep,
    page: usize,
) -> Option<usize> {
    if len == 0 {
        return None;
    }
    Some(match step {
        ui::NavigationStep::Next => index.map_or(0, |i| (i + 1).min(len - 1)),
        ui::NavigationStep::Previous => index.map_or(len - 1, |i| i.saturating_sub(1)),
        ui::NavigationStep::First => 0,
        ui::NavigationStep::Last => len - 1,
        ui::NavigationStep::PageNext => index.unwrap_or(0).saturating_add(page).min(len - 1),
        ui::NavigationStep::PagePrevious => index.unwrap_or(len - 1).saturating_sub(page),
    })
}
impl Catalogue {
    fn new(window: &ui::Showcase) -> Self {
        let currency = kit::CurrencyMetadata::cad().code;
        let products = (0..10_000_u32)
            .map(|i| Product {
                id: ProductId(i + 1),
                title: format!(
                    "{} {:05}",
                    [
                        "Field notebook",
                        "Studio collection",
                        "Daily journal",
                        "Desk companion"
                    ][i as usize % 4],
                    i + 1
                ),
                category: ["Stationery", "Collections", "Books", "Accessories"][i as usize % 4],
                price: kit::Money {
                    amount: kit::DecimalValue::parse_canonical(&format!(
                        "{}.{:02}",
                        8 + i % 140,
                        i % 100
                    ))
                    .expect("bounded exact example"),
                    currency: currency.clone(),
                },
                stock: i * 7 % 200,
            })
            .collect();
        let global = window.global::<ui::DataExamples>();
        let images = [
            global.get_thumbnail_one(),
            global.get_thumbnail_two(),
            global.get_thumbnail_three(),
        ];
        let articles:Vec<_>=(0..500).map(|i|ui::ContentItem {token:i+1,title:format!("{} · {}",["Objects with a purpose","Inside the studio","Notes from the field"][i as usize%3],i+1).into(),excerpt:"Thoughtful materials, small discoveries, and the people who bring an idea to life.".into(),metadata:format!("{} · {} min read · Collection {}",["Design","Journal","People"][i as usize%3],3+i%8,1+i/12).into(),link_label:"Read story".into(),thumbnail:images[i as usize%3].clone(),selected:false,disabled:false}).collect();
        let lines: Vec<_> = (0..50_000)
            .map(|i| ui::TextLine {
                token: i + 1,
                number: i + 1,
                text: format!(
                    "Record {:05} — Notes, values and source text remain in the document model.",
                    i + 1
                )
                .into(),
                selected: false,
            })
            .collect();
        Self {
            products,
            visible: vec![],
            selected: None,
            column: Column::Title,
            order: Order::Original,
            generation: 1,
            rows: Rc::new(VecModel::default()),
            articles: Rc::new(VecModel::from(articles)),
            article: None,
            lines: Rc::new(VecModel::from(lines)),
            line: None,
            drafts: BTreeMap::new(),
            line_generation: 1,
        }
    }
    fn rebuild(&mut self, query: &str) {
        let query = query.trim().to_lowercase();
        self.visible = self
            .products
            .iter()
            .enumerate()
            .filter(|(_, p)| {
                p.title.to_lowercase().contains(&query)
                    || p.category.to_lowercase().contains(&query)
            })
            .map(|(i, _)| i)
            .collect();
        if self.order != Order::Original {
            let products = &self.products;
            let column = self.column;
            let descending = self.order == Order::Descending;
            self.visible.sort_by(|a, b| {
                let a = &products[*a];
                let b = &products[*b];
                let result = match column {
                    Column::Title => a.title.cmp(&b.title),
                    Column::Category => a.category.cmp(b.category),
                    Column::Price => a.price.amount.cmp(&b.price.amount),
                    Column::Stock => a.stock.cmp(&b.stock),
                }
                .then(a.id.0.cmp(&b.id.0));
                if descending { result.reverse() } else { result }
            });
        }
        self.generation = self.generation.checked_add(1).expect("demo generation");
        let currency = kit::CurrencyMetadata::cad();
        self.rows.set_vec(
            self.visible
                .iter()
                .map(|index| {
                    let p = &self.products[*index];
                    ui::TableRow {
                        token: p.id.0 as i32,
                        cells: ModelRc::new(VecModel::from(vec![
                            p.title.as_str().into(),
                            p.category.into(),
                            p.price
                                .format(
                                    &currency,
                                    kit::NumberLocale::EnCa,
                                    kit::RoundingPolicy::Reject,
                                )
                                .expect("exact two decimals")
                                .into(),
                            p.stock.to_string().into(),
                        ])),
                        selected: self.selected == Some(p.id),
                        disabled: false,
                    }
                })
                .collect::<Vec<_>>(),
        );
    }
    fn publish(&self, window: &ui::Showcase) {
        let global = window.global::<ui::DataExamples>();
        global.set_table_generation(self.generation);
        global.set_table_active(self.active().map_or(-1, |i| i as i32));
        global.set_table_status(
            format!(
                "{} matching products · {}",
                self.visible.len(),
                self.selected.map_or_else(
                    || "No selection".into(),
                    |id| format!("Selected product #{}", id.0)
                )
            )
            .into(),
        );
        global.set_columns(ModelRc::new(VecModel::from(
            [
                Column::Title,
                Column::Category,
                Column::Price,
                Column::Stock,
            ]
            .iter()
            .enumerate()
            .map(|(i, column)| ui::TableColumn {
                token: i as i32 + 1,
                label: match column {
                    Column::Title => "Product",
                    Column::Category => "Category",
                    Column::Price => "Price · CAD",
                    Column::Stock => "In stock",
                }
                .into(),
                sortable: true,
                sort: if *column != self.column {
                    ui::DataSortDirection::Unsorted
                } else {
                    match self.order {
                        Order::Original => ui::DataSortDirection::Unsorted,
                        Order::Ascending => ui::DataSortDirection::Ascending,
                        Order::Descending => ui::DataSortDirection::Descending,
                    }
                },
                alignment: match column {
                    Column::Price | Column::Stock => ui::DataAlignment::Trailing,
                    Column::Title | Column::Category => ui::DataAlignment::Leading,
                },
            })
            .collect::<Vec<_>>(),
        )));
    }
    fn active(&self) -> Option<usize> {
        self.visible
            .iter()
            .position(|index| Some(self.products[*index].id) == self.selected)
    }
    fn select(&mut self, index: usize) {
        if let Some(old) = self.active() {
            let mut row = self.rows.row_data(old).unwrap();
            row.selected = false;
            self.rows.set_row_data(old, row);
        }
        let mut row = self.rows.row_data(index).unwrap();
        row.selected = true;
        self.selected = Some(self.products[self.visible[index]].id);
        self.rows.set_row_data(index, row);
    }
    fn choose_line(&mut self, index: usize, window: &ui::Showcase) {
        if let Some(old) = self.line {
            let mut row = self.lines.row_data(old).unwrap();
            row.selected = false;
            self.lines.set_row_data(old, row);
        }
        let mut row = self.lines.row_data(index).unwrap();
        row.selected = true;
        self.lines.set_row_data(index, row.clone());
        self.line = Some(index);
        let global = window.global::<ui::DataExamples>();
        global.set_line_active(index as i32);
        global.set_line_draft(row.text);
        global.set_line_status(format!("Line {} of 50,000", index + 1).into());
    }
}
pub fn install(window: &ui::Showcase) {
    let mut data = Catalogue::new(window);
    data.rebuild("");
    let global = window.global::<ui::DataExamples>();
    global.set_table_rows(model(data.rows.clone()));
    global.set_articles(model(data.articles.clone()));
    global.set_lines(model(data.lines.clone()));
    data.publish(window);
    let state = Rc::new(RefCell::new(data));
    let shared = state.clone();
    let weak = window.as_weak();
    global.on_query_edited(move |query| {
        if let Some(window) = weak.upgrade() {
            let mut data = shared.borrow_mut();
            data.rebuild(&query);
            data.publish(&window);
        }
    });
    let shared = state.clone();
    let weak = window.as_weak();
    global.on_sort(move |token| {
        let column = match token {
            1 => Column::Title,
            2 => Column::Category,
            3 => Column::Price,
            4 => Column::Stock,
            _ => return,
        };
        if let Some(window) = weak.upgrade() {
            let mut data = shared.borrow_mut();
            data.order = if data.column == column {
                match data.order {
                    Order::Original => Order::Ascending,
                    Order::Ascending => Order::Descending,
                    Order::Descending => Order::Original,
                }
            } else {
                Order::Ascending
            };
            data.column = column;
            data.rebuild(&window.global::<ui::DataExamples>().get_query());
            data.publish(&window);
        }
    });
    let shared = state.clone();
    let weak = window.as_weak();
    global.on_table_select(move |token, generation| {
        if let Some(window) = weak.upgrade() {
            let mut data = shared.borrow_mut();
            if data.generation != generation {
                return;
            }
            if let Some(index) = data
                .visible
                .iter()
                .position(|index| data.products[*index].id.0 as i32 == token)
            {
                data.select(index);
                data.publish(&window);
            }
        }
    });
    let shared = state.clone();
    let weak = window.as_weak();
    global.on_article_open(move |token, generation| {
        if generation != 1 {
            return;
        }
        if let Some(window) = weak.upgrade() {
            let mut data = shared.borrow_mut();
            let Some(index) = number(token, data.articles.row_count()) else {
                return;
            };
            if let Some(old) = data.article {
                let mut row = data.articles.row_data(old).unwrap();
                row.selected = false;
                data.articles.set_row_data(old, row);
            }
            let mut row = data.articles.row_data(index).unwrap();
            row.selected = true;
            data.articles.set_row_data(index, row.clone());
            data.article = Some(index);
            let global = window.global::<ui::DataExamples>();
            global.set_article_active(index as i32);
            global.set_article_status(
                format!("{}\n{}\n{}", row.title, row.excerpt, row.metadata).into(),
            );
        }
    });
    let shared = state.clone();
    let weak = window.as_weak();
    global.on_line_select(move |token, generation| {
        if let Some(window) = weak.upgrade() {
            let mut data = shared.borrow_mut();
            if generation == data.line_generation
                && let Some(index) = number(token, data.lines.row_count())
            {
                data.choose_line(index, &window);
            }
        }
    });
    let shared = state.clone();
    let weak = window.as_weak();
    global.on_line_commit(move |token, generation, text| {
        if let Some(window) = weak.upgrade() {
            let mut data = shared.borrow_mut();
            let Some(index) = number(token, data.lines.row_count()) else {
                return;
            };
            if generation != data.line_generation || data.line != Some(index) {
                return;
            }
            if text.contains(['\n', '\r']) {
                window.global::<ui::DataExamples>().set_line_status(
                    "Edit one line at a time; embedded newlines are not accepted.".into(),
                );
                return;
            }
            let mut row = data.lines.row_data(index).unwrap();
            row.text = text.clone();
            data.lines.set_row_data(index, row);
            data.drafts.insert(index, text.to_string());
            data.line_generation = data
                .line_generation
                .checked_add(1)
                .expect("document generation exhausted");
            window
                .global::<ui::DataExamples>()
                .set_line_generation(data.line_generation);
            window.global::<ui::DataExamples>().set_line_status(
                format!(
                    "Line {} saved. The other 49,999 lines are unchanged.",
                    index + 1
                )
                .into(),
            );
        }
    });
    let shared = state.clone();
    let weak = window.as_weak();
    global.on_line_copy(move |token, generation| {
        if let Some(window) = weak.upgrade() {
            let data = shared.borrow();
            if generation == data.line_generation
                && let Some(index) = number(token, data.lines.row_count())
            {
                window
                    .global::<ui::DataExamples>()
                    .set_line_status(format!("Line {} copied.", index + 1).into());
            }
        }
    });
    let weak = window.as_weak();
    global.on_navigate(move |collection, step| {
        let Some(window) = weak.upgrade() else {
            return;
        };
        let global = window.global::<ui::DataExamples>();
        let mut data = state.borrow_mut();
        match collection {
            ui::DataCollection::Table => {
                if let Some(index) = navigate(data.active(), data.visible.len(), step, 7) {
                    data.select(index);
                    data.publish(&window);
                }
            }
            ui::DataCollection::Articles => {
                if let Some(index) = navigate(data.article, data.articles.row_count(), step, 3) {
                    drop(data);
                    global.invoke_article_open(index as i32 + 1, 1);
                }
            }
            ui::DataCollection::Document => {
                if let Some(index) = navigate(data.line, data.lines.row_count(), step, 10) {
                    data.choose_line(index, &window);
                }
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sorting_filtering_and_line_edits_preserve_record_identity_and_unrelated_data() {
        i_slint_backend_testing::init_no_event_loop();
        let window = ui::Showcase::new().unwrap();
        install(&window);
        let data = window.global::<ui::DataExamples>();
        let original_generation = data.get_table_generation();
        data.invoke_table_select(17, original_generation);
        data.invoke_sort(3);
        let selected = data
            .get_table_rows()
            .row_data(data.get_table_active() as usize)
            .unwrap();
        assert_eq!(selected.token, 17, "sorting follows the selected record");
        data.invoke_table_select(1, original_generation);
        assert_eq!(
            data.get_table_rows()
                .row_data(data.get_table_active() as usize)
                .unwrap()
                .token,
            17
        );
        data.set_query("Accessories".into());
        data.invoke_query_edited("Accessories".into());
        assert_eq!(data.get_table_active(), -1);
        assert_eq!(data.get_table_rows().row_count(), 2500);
        data.set_query("".into());
        data.invoke_query_edited("".into());
        assert_eq!(
            data.get_table_rows()
                .row_data(data.get_table_active() as usize)
                .unwrap()
                .token,
            17
        );
        let untouched = data.get_lines().row_data(0).unwrap().text;
        let generation = data.get_line_generation();
        data.invoke_line_select(50_000, generation);
        data.invoke_line_commit(50_000, generation, "Dernière ligne — été".into());
        assert_eq!(
            data.get_lines().row_data(49_999).unwrap().text.as_str(),
            "Dernière ligne — été"
        );
        assert_eq!(data.get_lines().row_data(0).unwrap().text, untouched);
        data.invoke_line_commit(50_000, generation, "stale".into());
        assert_eq!(
            data.get_lines().row_data(49_999).unwrap().text.as_str(),
            "Dernière ligne — été"
        );
    }
}
