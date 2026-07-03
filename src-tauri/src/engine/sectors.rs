use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::models::Contract;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LiquidityTier {
    Core,
    Watch,
    Excluded,
}

impl std::str::FromStr for LiquidityTier {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.to_lowercase().as_str() {
            "watch" => Self::Watch,
            "excluded" => Self::Excluded,
            _ => Self::Core,
        })
    }
}

impl LiquidityTier {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Core => "core",
            Self::Watch => "watch",
            Self::Excluded => "excluded",
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FutureProduct {
    pub code: String,
    pub symbol: String,
    pub name: String,
    pub exchange: String,
    pub default_tier: LiquidityTier,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProductSector {
    pub code: String,
    pub name: String,
    pub jin10_category_id: Option<i64>,
    pub description: String,
    pub products: Vec<FutureProduct>,
    pub drivers: Vec<String>,
    pub news_keywords: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProductView {
    pub code: String,
    pub symbol: String,
    pub name: String,
    pub exchange: String,
    pub liquidity_tier: String,
    pub liquidity_score: Option<f64>,
    pub volume_20d: Option<f64>,
    pub turnover_20d: Option<f64>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SectorView {
    pub code: String,
    pub name: String,
    pub description: String,
    pub jin10_category_id: Option<i64>,
    pub drivers: Vec<String>,
    pub products: Vec<ProductView>,
}

fn sector(
    code: &str,
    name: &str,
    jin10: i64,
    desc: &str,
    products: &[(&str, &str, &str, &str, LiquidityTier)],
    drivers: &[&str],
    keywords: &[&str],
) -> ProductSector {
    ProductSector {
        code: code.into(),
        name: name.into(),
        jin10_category_id: Some(jin10),
        description: desc.into(),
        products: products
            .iter()
            .map(|(c, s, n, e, tier)| FutureProduct {
                code: (*c).into(),
                symbol: (*s).into(),
                name: (*n).into(),
                exchange: (*e).into(),
                default_tier: *tier,
            })
            .collect(),
        drivers: drivers.iter().map(|s| (*s).into()).collect(),
        news_keywords: keywords.iter().map(|s| (*s).into()).collect(),
    }
}

fn sectors_data() -> Vec<ProductSector> {
    let core = LiquidityTier::Core;
    let watch = LiquidityTier::Watch;
    vec![
        sector(
            "black",
            "黑色建材",
            52018,
            "钢材、炉料、煤焦和建材链，核心看地产/基建、钢厂利润和成材库存。",
            &[
                ("rb", "RB0", "螺纹钢", "SHFE", core),
                ("hc", "HC0", "热卷", "SHFE", core),
                ("i", "I0", "铁矿石", "DCE", core),
                ("j", "J0", "焦炭", "DCE", core),
                ("jm", "JM0", "焦煤", "DCE", core),
                ("fg", "FG0", "玻璃", "CZCE", watch),
                ("sa", "SA0", "纯碱", "CZCE", watch),
            ],
            &[
                "地产与基建需求",
                "钢厂利润",
                "铁水产量",
                "港口库存",
                "双焦供给",
            ],
            &[
                "钢材", "铁矿", "焦煤", "焦炭", "螺纹", "热卷", "玻璃", "纯碱",
            ],
        ),
        sector(
            "metals",
            "有色贵金属",
            52019,
            "基本金属、贵金属和新能源材料，核心看美元利率、库存和产业供需。",
            &[
                ("cu", "CU0", "沪铜", "SHFE", core),
                ("al", "AL0", "沪铝", "SHFE", core),
                ("zn", "ZN0", "沪锌", "SHFE", core),
                ("ni", "NI0", "沪镍", "SHFE", core),
                ("ao", "AO0", "氧化铝", "SHFE", watch),
                ("lc", "LC0", "碳酸锂", "GFEX", core),
                ("au", "AU0", "黄金", "SHFE", core),
                ("ag", "AG0", "白银", "SHFE", core),
            ],
            &[
                "美元与实际利率",
                "LME/国内库存",
                "冶炼利润",
                "新能源需求",
                "地缘避险",
            ],
            &[
                "铜",
                "铝",
                "锌",
                "镍",
                "锡",
                "黄金",
                "白银",
                "碳酸锂",
                "工业硅",
            ],
        ),
        sector(
            "agriculture",
            "农产品软商品",
            52034,
            "油脂油料、谷物、棉糖和养殖链，核心看天气、进口、库存和消费。",
            &[
                ("m", "M0", "豆粕", "DCE", core),
                ("y", "Y0", "豆油", "DCE", core),
                ("p", "P0", "棕榈油", "DCE", core),
                ("c", "C0", "玉米", "DCE", core),
                ("sr", "SR0", "白糖", "CZCE", core),
                ("cf", "CF0", "棉花", "CZCE", core),
                ("ap", "AP0", "苹果", "CZCE", watch),
                ("lh", "LH0", "生猪", "DCE", core),
            ],
            &["产区天气", "进口到港", "压榨利润", "库存消费比", "养殖利润"],
            &[
                "大豆",
                "豆粕",
                "豆油",
                "棕榈油",
                "玉米",
                "棉花",
                "白糖",
                "生猪",
            ],
        ),
        sector(
            "energy_chemical",
            "能源化工",
            52035,
            "油气、煤化工、聚酯、塑化和橡胶链，核心看原油、装置开工和库存。",
            &[
                ("sc", "SC0", "原油", "INE", core),
                ("fu", "FU0", "燃料油", "SHFE", core),
                ("bu", "BU0", "沥青", "SHFE", core),
                ("ta", "TA0", "PTA", "CZCE", core),
                ("ma", "MA0", "甲醇", "CZCE", core),
                ("pp", "PP0", "聚丙烯", "DCE", core),
                ("ru", "RU0", "橡胶", "SHFE", core),
                ("ur", "UR0", "尿素", "CZCE", watch),
            ],
            &[
                "原油价格",
                "装置开工率",
                "化工库存",
                "下游利润",
                "进出口窗口",
            ],
            &[
                "原油", "燃油", "沥青", "甲醇", "PTA", "橡胶", "尿素", "纸浆",
            ],
        ),
        sector(
            "shipping",
            "航运运价",
            52036,
            "集运指数及航运链，核心看地缘扰动、运力供给和外贸需求。",
            &[("ec", "EC0", "集运欧线", "INE", core)],
            &["红海/霍尔木兹扰动", "船舶绕航", "舱位供给", "欧美进口需求"],
            &["集运", "欧线", "航运", "运价", "红海", "霍尔木兹"],
        ),
    ]
}

static SECTORS: once_cell::sync::Lazy<Vec<ProductSector>> =
    once_cell::sync::Lazy::new(sectors_data);

static PRODUCT_TO_SECTOR: once_cell::sync::Lazy<HashMap<String, ProductSector>> =
    once_cell::sync::Lazy::new(|| {
        let mut map = HashMap::new();
        for s in SECTORS.iter() {
            for p in &s.products {
                map.insert(p.code.clone(), s.clone());
            }
        }
        map
    });

pub fn all_sectors() -> &'static [ProductSector] {
    &SECTORS
}

pub fn all_products() -> Vec<FutureProduct> {
    SECTORS.iter().flat_map(|s| s.products.clone()).collect()
}

/// 全部 core 商品（行情轮询 / 定时分析 / K 线回填）。
pub fn core_product_symbols() -> Vec<String> {
    all_products()
        .into_iter()
        .filter(|p| p.default_tier == LiquidityTier::Core)
        .map(|p| p.symbol.to_lowercase())
        .collect()
}

pub fn normalize_product(symbol: &str) -> String {
    let re = regex::Regex::new(r"(?i)^([a-z]+)").unwrap();
    re.captures(symbol.trim())
        .map(|c| c[1].to_lowercase())
        .unwrap_or_else(|| symbol.trim().to_lowercase())
}

pub fn get_sector_by_symbol(symbol: &str) -> Option<ProductSector> {
    PRODUCT_TO_SECTOR.get(&normalize_product(symbol)).cloned()
}

pub fn get_product_by_symbol(symbol: &str) -> Option<FutureProduct> {
    get_sector_by_symbol(symbol).and_then(|s| {
        s.products
            .iter()
            .find(|p| p.code == normalize_product(symbol))
            .cloned()
    })
}

pub fn sector_context(symbol: &str) -> serde_json::Value {
    let product = normalize_product(symbol);
    let sector = get_sector_by_symbol(&product);
    let product_meta = get_product_by_symbol(&product);
    if sector.is_none() {
        return serde_json::json!({
            "product": product,
            "product_name": product,
            "main_symbol": symbol,
            "name": "未分类",
            "related_products": [],
            "drivers": [],
            "news_keywords": [],
            "jin10_category_id": null,
        });
    }
    let sector = sector.unwrap();
    let related: Vec<serde_json::Value> = sector
        .products
        .iter()
        .filter(|p| p.code != product)
        .take(8)
        .map(|p| {
            serde_json::json!({
                "code": p.code,
                "symbol": p.symbol,
                "name": p.name,
            })
        })
        .collect();
    serde_json::json!({
        "product": product,
        "product_name": product_meta.as_ref().map(|p| p.name.clone()).unwrap_or(product.clone()),
        "main_symbol": product_meta.as_ref().map(|p| p.symbol.clone()).unwrap_or_else(|| symbol.to_string()),
        "code": sector.code,
        "name": sector.name,
        "description": sector.description,
        "related_products": related,
        "drivers": sector.drivers,
        "news_keywords": sector.news_keywords,
        "jin10_category_id": sector.jin10_category_id,
    })
}

pub fn all_contracts() -> Vec<Contract> {
    SECTORS
        .iter()
        .flat_map(|s| {
            s.products.iter().map(|p| Contract {
                symbol: p.symbol.clone(),
                exchange: p.exchange.clone(),
                name: p.name.clone(),
                product: p.code.clone(),
                multiplier: 10.0,
                margin_ratio: 0.1,
                listing_date: None,
                expiry_date: None,
            })
        })
        .collect()
}

pub fn default_news_category_ids() -> Vec<i64> {
    let mut ids: Vec<i64> = SECTORS.iter().filter_map(|s| s.jin10_category_id).collect();
    ids.push(52042);
    ids.sort_unstable();
    ids.dedup();
    ids
}

pub fn build_catalog(
    tier_filter: &str,
    liquidity: &HashMap<String, crate::models::LiquiditySnapshot>,
) -> Vec<SectorView> {
    SECTORS
        .iter()
        .filter_map(|sector| {
            let products: Vec<ProductView> = sector
                .products
                .iter()
                .filter_map(|p| {
                    let resolved = liquidity.get(&p.symbol.to_uppercase());
                    let tier = resolved
                        .map(|s| s.tier.clone())
                        .unwrap_or_else(|| p.default_tier.as_str().into());

                    let include = match tier_filter {
                        "all" => tier != "excluded",
                        "watch" => tier == "watch",
                        _ => tier == "core",
                    };
                    if !include {
                        return None;
                    }

                    Some(ProductView {
                        code: p.code.clone(),
                        symbol: p.symbol.clone(),
                        name: p.name.clone(),
                        exchange: p.exchange.clone(),
                        liquidity_tier: tier,
                        liquidity_score: resolved.map(|s| s.score),
                        volume_20d: resolved.map(|s| s.volume_20d),
                        turnover_20d: resolved.map(|s| s.turnover_20d),
                    })
                })
                .collect();

            if products.is_empty() {
                None
            } else {
                Some(SectorView {
                    code: sector.code.clone(),
                    name: sector.name.clone(),
                    description: sector.description.clone(),
                    jin10_category_id: sector.jin10_category_id,
                    drivers: sector.drivers.clone(),
                    products,
                })
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn core_product_count() {
        let core: usize = all_products()
            .iter()
            .filter(|p| p.default_tier == LiquidityTier::Core)
            .count();
        assert!(
            core >= 26,
            "expected 26+ core commodity products, got {core}"
        );
    }

    #[test]
    fn shipping_sector_present() {
        assert!(all_sectors().iter().any(|s| s.code == "shipping"));
        assert!(get_product_by_symbol("EC0").is_some());
    }
}
