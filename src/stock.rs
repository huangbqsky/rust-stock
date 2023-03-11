#![allow(dead_code)]

use std::{
    collections::HashMap,
    fs,
    io::Stdout,
    sync::{Arc, Mutex},
    thread,
};

use chrono::{DateTime, Local}; // 时间 library
use http_req::request; // 网络 library
use serde::{Deserialize, Serialize}; // 序列化 library
use serde_json::{json, Map, Value}; // 序列化 library
/**
tui：一款超好用的跨平台命令行界面库
使用 tui.rs 提供的以下模块进行 UI 编写(所有 UI 元素都实现了 Widget 或 StatefuWidget Trait)：

bakend 用于生成管理命令行的后端
layout 用于管理 UI 组件的布局
style 用于为 UI 添加样式
symbols 描述绘制散点图时所用点的样式
text 用于描述带样式的文本
widgets 包含预定义的 UI 组件

项目地址：https://github.com/fdehau/tui-rs
官方文档：https://docs.rs/tui/latest/tui/index.html

tui介绍：https://www.51cto.com/article/703696.html
实时股票数据： https://github.com/tarkah/tickrs
文件传输工具：https://github.com/veeso/termscp
网络监控工具：https://github.com/imsnif/bandwhich

 */
use tui::{backend::CrosstermBackend, widgets::ListState}; // UI library

pub type DynResult = Result<(), Box<dyn std::error::Error>>; // Ruesut的类型别名， Err是Trait Object 属于动态类型
pub type CrossTerminal = tui::Terminal<CrosstermBackend<Stdout>>; // 终端后端的类型别名，
pub type TerminalFrame<'a> = tui::Frame<'a, CrosstermBackend<Stdout>>; // 终端Frame的类型别名，

pub const DB_PATH: &str = ".stocks.json"; // 模拟数据库

// 股票信息结构体
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Stock {
    pub title: String,  // 股票名称
    pub code: String,   // 股票代码
    pub price: f64,     // 股票价格
    pub percent: f64,   // 股票涨跌
    pub open: f64,      // 今开
    pub yestclose: f64, // 昨收
    pub high: f64,      // 最高
    pub low: f64,       // 最低
}

impl Stock {
    pub fn new(code: &String) -> Self {
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
        app.load_stocks().unwrap_or_default();
        app.refresh_stocks();
        app
    }
    pub fn save_stocks(&self) -> DynResult {
        let db = dirs_next::home_dir().unwrap().join(DB_PATH);
        // 每个stock单独存一个对象，是考虑将来的扩展性
        let stocks = self.stocks.lock().unwrap();
        let lists: Vec<_> = stocks
            .iter()
            .map(|s| HashMap::from([("code", &s.code)]))
            .collect();
        fs::write(
            &db,
            serde_json::to_string(&HashMap::from([("stocks", lists)]))?,
        )?;
        Ok(())
    }

    pub fn load_stocks(&mut self) -> DynResult {
        // 用unwrap_or_default屏蔽文件不存在时的异常
        let content =
            fs::read_to_string(dirs_next::home_dir().unwrap().join(DB_PATH)).unwrap_or_default();
        // 如果直接转换stocks，必须所有key都对上, 兼容性不好
        // self.stocks = serde_json::from_str(&content).unwrap_or_default();

        // 先读成Map再转换，可以增加兼容性，
        let json: Map<String, Value> = serde_json::from_str(&content).unwrap_or_default();
        let mut data = self.stocks.lock().unwrap();
        data.clear();
        data.append(
            &mut json
                .get("stocks")
                .unwrap_or(&json!([]))
                .as_array()
                .unwrap()
                .iter()
                .map(|s| {
                    Stock::new(
                        &s.as_object()
                            .unwrap()
                            .get("code")
                            .unwrap()
                            .as_str()
                            .unwrap()
                            .to_string(),
                    )
                })
                .collect(),
        );

        Ok(())
    }

    pub fn refresh_stocks(&mut self) {
        let stock_clone = self.stocks.clone();
        let err_clone = self.error.clone();
        let last_refresh_clone = self.last_refresh.clone();
        let codes = self.get_codes();
        if codes.len() > 0 {
            thread::spawn(move || {
                let mut writer = Vec::new();
                let ret = request::get(
                    format!("{}{}", "http://api.money.126.net/data/feed/", codes),
                    &mut writer,
                );
                let mut locked_err = err_clone.lock().unwrap();
                if let Err(err) = ret {
                    *locked_err = format!("{:?}", err);
                } else {
                    let content = String::from_utf8_lossy(&writer);
                    if content.starts_with("_ntes_quote_callback") {
                        let mut stocks = stock_clone.lock().unwrap();
                        //网易的返回包了一个js call，用skip,take,collect实现一个substring剥掉它
                        let json: Map<String, Value> = serde_json::from_str(
                            &content
                                .chars()
                                .skip(21)
                                .take(content.len() - 23)
                                .collect::<String>(),
                        )
                        .unwrap();
                        for stock in stocks.iter_mut() {
                            //如果code不对,返回的json里不包括这个对象, 用unwrap_or生成一个空对象,防止异常
                            let obj = json
                                .get(&stock.code)
                                .unwrap_or(&json!({}))
                                .as_object()
                                .unwrap()
                                .to_owned();
                            stock.title = obj
                                .get("name")
                                .unwrap_or(&json!(stock.code.clone()))
                                .as_str()
                                .unwrap()
                                .to_owned();
                            stock.price = obj.get("price").unwrap_or(&json!(0.0)).as_f64().unwrap();
                            stock.percent =
                                obj.get("percent").unwrap_or(&json!(0.0)).as_f64().unwrap();
                            stock.open = obj.get("open").unwrap_or(&json!(0.0)).as_f64().unwrap();
                            stock.yestclose = obj
                                .get("yestclose")
                                .unwrap_or(&json!(0.0))
                                .as_f64()
                                .unwrap();
                            stock.high = obj.get("high").unwrap_or(&json!(0.0)).as_f64().unwrap();
                            stock.low = obj.get("low").unwrap_or(&json!(0.0)).as_f64().unwrap();

                            // if json.contains_key(&stock.code) {
                            //     let mut writer2 = Vec::new();
                            //     request::get(format!("http://img1.money.126.net/data/hs/time/today/{}.json",stock.code), &mut writer2)?;
                            //     println!("{:?}", format!("http://img1.money.126.net/data/hs/time/today/{}.json",stock.code));
                            //     let json2: Map<String, Value> = serde_json::from_str(&String::from_utf8_lossy(&writer2).to_string())?;
                            //     stock.slice = json2.get("data").unwrap().as_array().unwrap()
                            //         .iter().map(|item| item.as_array().unwrap().get(2).unwrap().as_f64().unwrap())
                            //         .collect();
                            // }
                        }
                        let mut last_refresh = last_refresh_clone.lock().unwrap();
                        *last_refresh = Local::now();
                        *locked_err = String::new();
                    } else {
                        *locked_err = String::from("服务器返回错误");
                    }
                }
            });
        }
    }

    pub fn get_codes(&self) -> String {
        let codes: Vec<String> = self
            .stocks
            .lock()
            .unwrap()
            .iter()
            .map(|stock| stock.code.clone())
            .collect();
        codes.join(",")
    }
}
