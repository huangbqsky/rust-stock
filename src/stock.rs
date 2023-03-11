#![allow(dead_code)]

use std::{io::Stdout, fs, collections::HashMap, sync::{Mutex, Arc}, thread};

use chrono::{DateTime, Local};
use http_req::request;
use serde::{Serialize, Deserialize};
use serde_json::{Value, Map, json};
use tui::{backend::CrosstermBackend, widgets::ListState};


pub type DynResult = Result<(), Box<dyn std::error::Error>>;
pub type CrossTerminal = tui::Terminal<CrosstermBackend<Stdout>>;
pub type TerminalFrame<'a> = tui::Frame<'a, CrosstermBackend<Stdout>>;

pub const DB_PATH: &str = ".stocks.json";


pub mod widget;

// 股票信息结构体
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Stock {
    pub title: String, // 股票名称
    pub code: String, // 股票代码
    pub price: f64, // 股票价格
    pub percent: f64, // 股票涨跌
    pub open: f64, // 今开
    pub yestclose: f64, // 昨收
    pub high: f64, // 最高
    pub low: f64, // 最低
}

impl Stock {
    pub fn new (code: &String) -> Self {
        Self {
            code: code.clone(),
            title: code.clone(),
            price: 0.0,
            percent: 0.0,
            open: 0.0,
            yestclose: 0.0,
            high: 0.0,
            low: 0.0,
        }
    }
}

pub enum AppState {
    Normal,
    Adding,
}

pub struct App {
    pub should_exit: bool,
    pub state: AppState,
    pub error: Arc<Mutex<String>>,
    pub input: String,
    pub stocks: Arc<Mutex<Vec<Stock>>>,
     // TUI的List控件需要这个state记录当前选中和滚动位置两个状态
    pub stocks_state: ListState,
    pub last_refresh: Arc<Mutex<DateTime<Local>>>,
    pub tick_count: u128,
}

impl App {
    pub fn new() -> Self {
        let mut app = Self {
            should_exit: false,
            state: AppState::Normal,
            input: String::new(),
            error: Arc::new(Mutex::new(String::new())),
            stocks: Arc::new(Mutex::new([].to_vec())),
              //ListState:default为未选择，因为可能stocks为空，所以不能自动选第一个
            stocks_state: ListState::default(),
            last_refresh: Arc::new(Mutex::new(Local::now())),
            tick_count: 0,

        };

        app
    }
}