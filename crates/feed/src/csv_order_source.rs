use std::fs;
use std::path::Path;

use core::symbols::SymbolRegistry;
use core::{
    AlgoOrderRequest, ClientId, Message, OrderRequestAction, Side, StrategyType, TimeNanos,
};

use crate::MessageSource;

#[derive(Debug, Clone, PartialEq)]
pub enum CsvSourceError {
    Read(String),
    InvalidColumnCount {
        line: usize,
        found: usize,
    },
    InvalidField {
        line: usize,
        field: &'static str,
        value: String,
    },
    UnknownSymbol {
        line: usize,
        symbol: String,
    },
    InvalidSide {
        line: usize,
        value: i64,
    },
    InvalidQuantity {
        line: usize,
        value: f64,
    },
    InvalidAggressiveness {
        line: usize,
        value: i64,
    },
}

#[derive(Debug, Clone)]
pub struct OrderRequestCsvSource {
    messages: Vec<Message>,
    next_index: usize,
}

impl OrderRequestCsvSource {
    /// Loads rows in the format:
    /// `idx,timestamp_us,parent_order_id,symbol,side,size,duration_sec,aggressiveness`.
    pub fn from_path(
        path: impl AsRef<Path>,
        registry: &SymbolRegistry,
        client_id: ClientId,
    ) -> Result<Self, CsvSourceError> {
        let contents = fs::read_to_string(path.as_ref())
            .map_err(|error| CsvSourceError::Read(error.to_string()))?;
        Self::from_str(&contents, registry, client_id)
    }

    pub fn from_str(
        contents: &str,
        registry: &SymbolRegistry,
        client_id: ClientId,
    ) -> Result<Self, CsvSourceError> {
        let mut rows = Vec::new();
        for (line_index, raw_line) in contents.lines().enumerate() {
            let line = line_index + 1;
            let trimmed = raw_line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            let columns: Vec<&str> = trimmed.split(',').map(str::trim).collect();
            if is_header(&columns) {
                continue;
            }
            if columns.len() != 8 {
                return Err(CsvSourceError::InvalidColumnCount {
                    line,
                    found: columns.len(),
                });
            }

            let timestamp_us = parse_u64(columns[1], line, "timestamp_us")?;
            let parent_order_id = parse_u64(columns[2], line, "parent_order_id")?;
            let symbol_name = columns[3].to_string();
            let symbol = registry.get_by_flat_id(&symbol_name).ok_or_else(|| {
                CsvSourceError::UnknownSymbol {
                    line,
                    symbol: symbol_name,
                }
            })?;
            let side_value = parse_i64(columns[4], line, "side")?;
            let side = match side_value {
                1 => Side::Buy,
                -1 => Side::Sell,
                value => return Err(CsvSourceError::InvalidSide { line, value }),
            };
            let quantity = parse_f64(columns[5], line, "size")?;
            if !(quantity > 0.0) {
                return Err(CsvSourceError::InvalidQuantity {
                    line,
                    value: quantity,
                });
            }
            let duration_sec = parse_u64(columns[6], line, "duration_sec")?;
            let aggressiveness = parse_i64(columns[7], line, "aggressiveness")?;
            if !(1..=100).contains(&aggressiveness) {
                return Err(CsvSourceError::InvalidAggressiveness {
                    line,
                    value: aggressiveness,
                });
            }

            let start_time = timestamp_us as TimeNanos * 1_000;
            let end_time = start_time + duration_sec as TimeNanos * 1_000_000_000;
            rows.push((
                start_time,
                parent_order_id,
                Message::AlgoOrderRequest(AlgoOrderRequest {
                    client_id,
                    parent_order_id: core::ParentOrderId(parent_order_id),
                    symbol: symbol.id,
                    side,
                    qty: quantity,
                    start_time,
                    end_time,
                    strategy_type: StrategyType::Unknown,
                    action: OrderRequestAction::New,
                }),
            ));
        }
        rows.sort_by_key(|(timestamp, parent_order_id, _)| (*timestamp, *parent_order_id));
        Ok(Self {
            messages: rows.into_iter().map(|(_, _, message)| message).collect(),
            next_index: 0,
        })
    }
}

impl MessageSource for OrderRequestCsvSource {
    fn next_message(&mut self) -> Option<Message> {
        let message = self.messages.get(self.next_index).copied();
        self.next_index += usize::from(message.is_some());
        message
    }

    fn source_name(&self) -> &str {
        "ORDER_REQUEST_CSV"
    }
}

fn is_header(columns: &[&str]) -> bool {
    columns.first().is_some_and(|value| {
        matches!(
            value.to_ascii_lowercase().as_str(),
            "idx" | "index" | "order_id" | "parent_order_id"
        )
    }) || columns.get(1).is_some_and(|value| {
        matches!(
            value.to_ascii_lowercase().as_str(),
            "order_id" | "parent_order_id"
        )
    })
}

fn parse_u64(value: &str, line: usize, field: &'static str) -> Result<u64, CsvSourceError> {
    value.parse().map_err(|_| CsvSourceError::InvalidField {
        line,
        field,
        value: value.into(),
    })
}
fn parse_i64(value: &str, line: usize, field: &'static str) -> Result<i64, CsvSourceError> {
    value.parse().map_err(|_| CsvSourceError::InvalidField {
        line,
        field,
        value: value.into(),
    })
}
fn parse_f64(value: &str, line: usize, field: &'static str) -> Result<f64, CsvSourceError> {
    value.parse().map_err(|_| CsvSourceError::InvalidField {
        line,
        field,
        value: value.into(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::{AlgoOrderRequest, SymbolId};

    #[test]
    fn csv_source_loads_and_sorts_algo_orders() {
        let mut registry = SymbolRegistry::default();
        registry
            .register(core::symbols::Symbol::new(
                SymbolId(1),
                "BTC-USDT",
                "TEST",
                0.1,
                0.001,
            ))
            .unwrap();
        let csv = "idx,timestamp_us,parent_order_id,symbol,side,size,duration_sec,aggressiveness\n2,20,8,BTC-USDT,-1,2.0,3,50\n1,10,7,BTC-USDT,1,1.0,2,50\n";
        let mut source = OrderRequestCsvSource::from_str(csv, &registry, ClientId(9)).unwrap();
        let first = source.next_message().unwrap();
        assert!(matches!(
            first,
            Message::AlgoOrderRequest(AlgoOrderRequest {
                parent_order_id: core::ParentOrderId(7),
                ..
            })
        ));
        assert_eq!(source.next_message().unwrap().ts_nanos(), 20_000);
    }
}
