//! 分析维度字典与关键词（规则分类用）。

use std::collections::HashMap;

use once_cell::sync::Lazy;

#[derive(Clone, Debug)]
pub struct DimensionDef {
    pub code: &'static str,
    pub label: &'static str,
    pub keywords: &'static [&'static str],
}

const DIMENSIONS: &[DimensionDef] = &[
    DimensionDef {
        code: "seasonality",
        label: "季节性",
        keywords: &["旺季", "淡季", "检修", "开工", "冬储", "春播", "收割", "种植"],
    },
    DimensionDef {
        code: "weather",
        label: "天气",
        keywords: &["降雨", "干旱", "霜冻", "高温", "寒潮", "台风", "洪涝", "天气", "降水"],
    },
    DimensionDef {
        code: "overseas_upstream",
        label: "海外上游",
        keywords: &[
            "LME", "CBOT", "普氏", "发运", "到港", "进口", "外盘", "原油", "OPEC", "巴西", "澳洲",
        ],
    },
    DimensionDef {
        code: "domestic_supply",
        label: "国内供给",
        keywords: &["产量", "开工率", "检修", "复产", "限产", "减产", "供给", "供应", "产能"],
    },
    DimensionDef {
        code: "demand",
        label: "需求",
        keywords: &["需求", "消费", "销售", "订单", "地产", "基建", "开工率", "出口", "进口需求"],
    },
    DimensionDef {
        code: "inventory",
        label: "库存",
        keywords: &["库存", "仓单", "社会库存", "港口库存", "厂库", "累库", "去库"],
    },
    DimensionDef {
        code: "spread_arb",
        label: "价差套利",
        keywords: &["价差", "基差", "期现", "套利", "利润", "榨利", "进口利润"],
    },
    DimensionDef {
        code: "policy",
        label: "政策监管",
        keywords: &["政策", "关税", "收储", "抛储", "环保", "限产", "监管", "补贴"],
    },
    DimensionDef {
        code: "macro",
        label: "国内宏观",
        keywords: &[
            "PMI", "GDP", "LPR", "M2", "社融", "降准", "降息", "央行", "国务院", "国家统计局",
            "人民币", "通胀", "宏观", "股指", "A股", "中国CPI", "中国PPI", "财新", "制造业PMI",
        ],
    },
    DimensionDef {
        code: "overseas_finance",
        label: "国外金融环境",
        keywords: &[
            "美联储", "FOMC", "Fed", "鲍威尔", "Powell", "加息", "降息", "缩表", "点阵图",
            "CPI", "PPI", "PCE", "核心PCE", "非农", "NFP", "ISM", "初请", "零售销售",
            "美国通胀", "美国就业", "美国GDP", "美债", "10年期", "2年期", "美元", "美元指数",
            "DXY", "鹰派", "鸽派", "利率决策", "货币政策", "欧央行", "ECB", "日本央行",
        ],
    },
    DimensionDef {
        code: "geopolitics",
        label: "地缘",
        keywords: &["地缘", "制裁", "战争", "冲突", "红海", "霍尔木兹", "航道", "扰动"],
    },
    DimensionDef {
        code: "earnings",
        label: "企业财报",
        keywords: &["财报", "业绩", "盈利", "净利润", "指引", "季报", "年报"],
    },
    DimensionDef {
        code: "flow",
        label: "资金持仓",
        keywords: &["持仓", "净空", "净多", "主力", "资金", "多头", "空头"],
    },
];

static SECTOR_DIMS: Lazy<HashMap<&'static str, Vec<&'static str>>> = Lazy::new(|| {
    HashMap::from([
        (
            "black",
            vec![
                "demand",
                "domestic_supply",
                "inventory",
                "overseas_finance",
                "overseas_upstream",
                "policy",
            ],
        ),
        (
            "metals",
            vec![
                "macro",
                "overseas_finance",
                "overseas_upstream",
                "inventory",
                "earnings",
                "geopolitics",
            ],
        ),
        (
            "agriculture",
            vec![
                "weather",
                "seasonality",
                "overseas_finance",
                "overseas_upstream",
                "inventory",
                "demand",
            ],
        ),
        (
            "energy_chemical",
            vec![
                "overseas_upstream",
                "overseas_finance",
                "domestic_supply",
                "inventory",
                "spread_arb",
                "policy",
            ],
        ),
        (
            "shipping",
            vec![
                "geopolitics",
                "demand",
                "overseas_finance",
                "seasonality",
                "overseas_upstream",
            ],
        ),
        (
            "financial",
            vec!["macro", "overseas_finance", "policy", "flow", "technical"],
        ),
    ])
});

static DIM_MAP: Lazy<HashMap<&'static str, DimensionDef>> = Lazy::new(|| {
    DIMENSIONS
        .iter()
        .map(|d| (d.code, d.clone()))
        .collect()
});

pub fn all_dimensions() -> &'static [DimensionDef] {
    DIMENSIONS
}

pub fn dimension_label(code: &str) -> &str {
    DIM_MAP
        .get(code)
        .map(|d| d.label)
        .unwrap_or(code)
}

pub fn sector_dimension_codes(sector_code: &str) -> Vec<&'static str> {
    SECTOR_DIMS
        .get(sector_code)
        .cloned()
        .unwrap_or_else(|| vec!["demand", "domestic_supply", "inventory"])
}

pub fn dimension_keywords(code: &str) -> Vec<&'static str> {
    DIM_MAP
        .get(code)
        .map(|d| d.keywords.to_vec())
        .unwrap_or_default()
}

pub fn seed_rows() -> Vec<(&'static str, &'static str, &'static str)> {
    DIMENSIONS
        .iter()
        .map(|d| (d.code, d.label, ""))
        .collect()
}
