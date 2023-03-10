#![allow(dead_code)]

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