#!/usr/bin/env python3
"""
股票数据同步脚本 - 供外部定时服务调用
"""

import os
import sys
import logging
from datetime import datetime

logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(levelname)s - %(message)s'
)
logger = logging.getLogger(__name__)

def main():
    logger.info("=" * 50)
    logger.info("开始股票数据同步任务")
    logger.info(f"执行时间: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}")
    logger.info("=" * 50)
    
    try:
        from akshare_data import StockDataCollector
        
        collector = StockDataCollector(
            output_dir="data/stocks",
            db_config={
                'host': os.getenv('DB_HOST', 'localhost'),
                'port': os.getenv('DB_PORT', '5432'),
                'database': os.getenv('DB_NAME', 'stock_replay'),
                'user': os.getenv('DB_USER', 'postgres'),
                'password': os.getenv('DB_PASSWORD', 'postgres'),
            }
        )
        
        count = collector.sync_all_stocks_to_db(batch_size=100)
        
        logger.info("=" * 50)
        logger.info(f"同步完成! 共同步 {count} 条股票数据")
        logger.info("=" * 50)
        
        return 0
        
    except Exception as e:
        logger.error(f"同步失败: {e}")
        return 1

if __name__ == "__main__":
    sys.exit(main())
