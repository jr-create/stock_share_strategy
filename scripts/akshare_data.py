#!/usr/bin/env python3
"""
AkShare 数据采集脚本
用于获取股票实时行情、历史数据、板块数据等
"""

import akshare as ak
import pandas as pd
import json
import os
import time
from datetime import datetime, timedelta
from typing import Optional, List, Dict, Any
import psycopg2
from psycopg2.extras import execute_batch
import argparse

class StockDataCollector:
    """股票数据采集器"""
    
    def __init__(self, output_dir: str = "data/stocks", db_config: Dict[str, str] = None):
        self.output_dir = output_dir
        os.makedirs(output_dir, exist_ok=True)
        self.db_config = db_config or self._get_db_config_from_env()
    
    def _get_db_config_from_env(self) -> Dict[str, str]:
        """从环境变量获取数据库配置"""
        return {
            'host': os.getenv('DB_HOST', 'localhost'),
            'port': os.getenv('DB_PORT', '5432'),
            'database': os.getenv('DB_NAME', 'stock_replay'),
            'user': os.getenv('DB_USER', 'postgres'),
            'password': os.getenv('DB_PASSWORD', 'postgres'),
        }
    
    def get_realtime_quotes(self) -> pd.DataFrame:
        """
        获取A股实时行情数据
        返回所有沪深京A股上市公司的实时行情
        """
        print("正在获取A股实时行情数据...")
        df = ak.stock_zh_a_spot_em()
        df['update_time'] = datetime.now().strftime('%Y-%m-%d %H:%M:%S')
        output_path = os.path.join(self.output_dir, "realtime_quotes.json")
        df.to_json(output_path, orient='records', force_ascii=False)
        print(f"实时行情数据已保存到 {output_path}, 共 {len(df)} 条记录")
        return df
    
    def get_stock_history(
        self, 
        symbol: str, 
        start_date: str = None, 
        end_date: str = None,
        period: str = "daily",
        adjust: str = "qfq"
    ) -> pd.DataFrame:
        """
        获取单只股票历史数据
        Args:
            symbol: 股票代码，如 "000001"
            start_date: 开始日期，格式 "20240101"
            end_date: 结束日期，格式 "20240131"
            period: 周期，可选 "daily"(日线), "weekly"(周线), "monthly"(月线)
            adjust: 复权类型，"qfq"(前复权), "hfq"(后复权), ""(不复权)
        """
        if start_date is None:
            start_date = (datetime.now() - timedelta(days=365)).strftime('%Y%m%d')
        if end_date is None:
            end_date = datetime.now().strftime('%Y%m%d')
        
        print(f"正在获取 {symbol} 历史数据 ({start_date} ~ {end_date})...")
        df = ak.stock_zh_a_hist(
            symbol=symbol,
            period=period,
            start_date=start_date,
            end_date=end_date,
            adjust=adjust
        )
        df['symbol'] = symbol
        output_path = os.path.join(self.output_dir, f"history_{symbol}.json")
        df.to_json(output_path, orient='records', force_ascii=False)
        print(f"历史数据已保存到 {output_path}")
        return df
    
    def get_index_history(
        self,
        symbol: str = "sh000001",
        start_date: str = None,
        end_date: str = None,
        period: str = "daily"
    ) -> pd.DataFrame:
        """
        获取指数历史数据
        
        Args:
            symbol: 指数代码，如 "sh000001"(上证指数), "sz399001"(深证成指)
            start_date: 开始日期
            end_date: 结束日期
            period: 周期
        """
        if start_date is None:
            start_date = (datetime.now() - timedelta(days=365)).strftime('%Y%m%d')
        if end_date is None:
            end_date = datetime.now().strftime('%Y%m%d')
        
        print(f"正在获取指数 {symbol} 历史数据...")
        df = ak.index_zh_a_hist(
            symbol=symbol,
            period=period,
            start_date=start_date,
            end_date=end_date
        )
        output_path = os.path.join(self.output_dir, f"index_{symbol}.json")
        df.to_json(output_path, orient='records', force_ascii=False)
        print(f"指数数据已保存到 {output_path}")
        return df
    
    def get_sector_data(self, indicator: str = "涨跌幅") -> pd.DataFrame:
        """
        获取板块数据
        
        Args:
            indicator: 排序指标，可选 "涨跌幅", "涨速", "5分钟涨跌", "成交量", "成交额" 等
        """
        print(f"正在获取板块数据 (排序: {indicator})...")
        df = ak.stock_board_industry_em(indicator=indicator)
        output_path = os.path.join(self.output_dir, "sectors.json")
        df.to_json(output_path, orient='records', force_ascii=False)
        print(f"板块数据已保存到 {output_path}")
        return df
    
    def get_sector_stocks(self, sector_name: str) -> pd.DataFrame:
        """
        获取板块成分股
        
        Args:
            sector_name: 板块名称，如 "软件开发"
        """
        print(f"正在获取板块 {sector_name} 成分股...")
        df = ak.stock_board_industry_cons_em(symbol=sector_name)
        output_path = os.path.join(self.output_dir, f"sector_{sector_name}_stocks.json")
        df.to_json(output_path, orient='records', force_ascii=False)
        print(f"板块成分股已保存到 {output_path}")
        return df
    
    def get_stock_fund_flow(self, symbol: str, market: str = "sh") -> pd.DataFrame:
        """
        获取个股资金流向
        
        Args:
            symbol: 股票代码
            market: 市场，"sh" 或 "sz"
        """
        print(f"正在获取 {symbol} 资金流向数据...")
        df = ak.stock_individual_fund_flow(stock=symbol, market=market)
        output_path = os.path.join(self.output_dir, f"fund_flow_{symbol}.json")
        df.to_json(output_path, orient='records', force_ascii=False)
        print(f"资金流向数据已保存到 {output_path}")
        return df
    
    def get_stock_info(self, symbol: str) -> Dict[str, Any]:
        """
        获取股票基本信息
        
        Args:
            symbol: 股票代码
        """
        print(f"正在获取 {symbol} 基本信息...")
        df = ak.stock_individual_info_em(symbol=symbol)
        info = dict(zip(df['item'], df['value']))
        output_path = os.path.join(self.output_dir, f"info_{symbol}.json")
        with open(output_path, 'w', encoding='utf-8') as f:
            json.dump(info, f, ensure_ascii=False, indent=2)
        print(f"基本信息已保存到 {output_path}")
        return info
    
    def get_hot_stocks(self) -> pd.DataFrame:
        """获取热门股票"""
        print("正在获取热门股票...")
        df = ak.stock_hot_rank_em()
        output_path = os.path.join(self.output_dir, "hot_stocks.json")
        df.to_json(output_path, orient='records', force_ascii=False)
        print(f"热门股票已保存到 {output_path}")
        return df
    
    def collect_all_basic_data(self):
        """采集所有基础数据"""
        print("=" * 50)
        print("开始采集股票基础数据")
        print("=" * 50)
        
        try:
            self.get_realtime_quotes()
            time.sleep(1)
        except Exception as e:
            print(f"获取实时行情失败: {e}")
        
        try:
            self.get_sector_data()
            time.sleep(1)
        except Exception as e:
            print(f"获取板块数据失败: {e}")
        
        try:
            self.get_hot_stocks()
            time.sleep(1)
        except Exception as e:
            print(f"获取热门股票失败: {e}")
        
        try:
            self.get_index_history("sh000001")
            time.sleep(1)
        except Exception as e:
            print(f"获取上证指数失败: {e}")
        
        print("=" * 50)
        print("数据采集完成")
        print("=" * 50)
    
    def sync_all_stocks_to_db(self, batch_size: int = 100) -> int:
        """
        获取所有A股实时行情并同步到数据库
        
        Args:
            batch_size: 批量插入的批次大小
            
        Returns:
            同步的股票数量
        """
        print("正在获取所有A股实时行情数据...")
        
        df = ak.stock_info_a_code_name()
        print(f"获取到 {len(df)} 条股票数据")
        print(f"列名: {df.columns.tolist()}")
        
        code_col = None
        name_col = None
        for col in df.columns:
            col_lower = str(col).lower()
            if '代码' in col or 'code' in col_lower:
                code_col = col
            if '名称' in col or 'name' in col_lower:
                name_col = col
        
        if code_col is None or name_col is None:
            print(f"错误: 无法识别列名。可用列: {df.columns.tolist()}")
            return 0
        
        print(f"使用列: 代码={code_col}, 名称={name_col}")
        
        stocks_data = []
        for _, row in df.iterrows():
            code = str(row.get(code_col, '')).strip()
            name = str(row.get(name_col, '')).strip()
            
            if not code or not name:
                continue
            
            market = self._determine_market(code)
            sector = str(row.get('板块', '')).strip() if '板块' in row else ''
            
            stocks_data.append({
                'code': code,
                'name': name,
                'market': market,
                'sector': sector,
            })
        
        print(f"预处理后有效数据 {len(stocks_data)} 条")
        
        # 连接数据库并同步
        try:
            conn = psycopg2.connect(**self.db_config)
            cursor = conn.cursor()
            
            # 使用 UPSERT 批量插入或更新
            insert_count = 0
            for i in range(0, len(stocks_data), batch_size):
                batch = stocks_data[i:i + batch_size]
                
                # 构建批量插入语句
                values = []
                for stock in batch:
                    values.append((
                        stock['code'],
                        stock['name'],
                        stock['market'],
                        stock['sector'],
                    ))
                
                execute_batch(cursor, """
                    INSERT INTO stocks (code, name, market, sector, updated_at)
                    VALUES (%s, %s, %s, %s, NOW())
                    ON CONFLICT (code) DO UPDATE SET
                        name = EXCLUDED.name,
                        market = COALESCE(NULLIF(EXCLUDED.market, ''), stocks.market),
                        sector = COALESCE(NULLIF(EXCLUDED.sector, ''), stocks.sector),
                        updated_at = NOW()
                """, values)
                
                insert_count += len(batch)
                print(f"已同步 {insert_count}/{len(stocks_data)} 条数据...")
            
            conn.commit()
            cursor.close()
            conn.close()
            
            print(f"✓ 成功同步 {insert_count} 条股票数据到数据库")
            return insert_count
            
        except Exception as e:
            print(f"✗ 数据库同步失败: {e}")
            raise
    
    def _determine_market(self, code: str) -> str:
        """
        根据股票代码判断所属市场
        
        Args:
            code: 股票代码
            
        Returns:
            市场名称（沪/深/北）
        """
        if not code:
            return ''
        
        # 上海市场：6开头
        if code.startswith('6'):
            return '沪'
        # 深圳市场：0、3开头
        elif code.startswith('0') or code.startswith('3'):
            return '深'
        # 北京市场：8开头
        elif code.startswith('8'):
            return '北'
        # 其他情况
        else:
            return ''


def main():
    parser = argparse.ArgumentParser(description="AkShare 股票数据采集工具")
    parser.add_argument("--output", "-o", default="data/stocks", help="输出目录")
    parser.add_argument("--realtime", action="store_true", help="获取实时行情")
    parser.add_argument("--history", type=str, help="获取指定股票历史数据")
    parser.add_argument("--index", type=str, help="获取指定指数历史数据")
    parser.add_argument("--sector", action="store_true", help="获取板块数据")
    parser.add_argument("--fund-flow", type=str, help="获取指定股票资金流向")
    parser.add_argument("--info", type=str, help="获取指定股票基本信息")
    parser.add_argument("--hot", action="store_true", help="获取热门股票")
    parser.add_argument("--all", action="store_true", help="采集所有基础数据")
    parser.add_argument("--sync-db", action="store_true", help="同步所有A股数据到数据库")
    parser.add_argument("--batch-size", type=int, default=100, help="数据库批量插入大小")
    
    args = parser.parse_args()
    
    collector = StockDataCollector(output_dir=args.output)
    
    if args.sync_db:
        collector.sync_all_stocks_to_db(batch_size=args.batch_size)
    elif args.all:
        collector.collect_all_basic_data()
    elif args.realtime:
        collector.get_realtime_quotes()
    elif args.history:
        collector.get_stock_history(args.history)
    elif args.index:
        collector.get_index_history(args.index)
    elif args.sector:
        collector.get_sector_data()
    elif args.fund_flow:
        collector.get_stock_fund_flow(args.fund_flow)
    elif args.info:
        collector.get_stock_info(args.info)
    elif args.hot:
        collector.get_hot_stocks()
    else:
        parser.print_help()


if __name__ == "__main__":
    main()
