//! 额度桶的纯逻辑：谁先被扣。不碰数据库，方便穷举测试。

use chrono::{DateTime, Utc};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BucketView {
    pub id: i64,
    pub remaining: i64,
    pub expires_at: Option<DateTime<Utc>>,
}

/// 把一段消耗分摊到各个桶：最早过期的先扣，同时过期则剩余少的先扣，没有过期时间的最后扣。
/// 返回每个桶的扣减量（只含扣减量大于 0 的桶），以及所有桶都不够时的透支量。
pub fn allocate(consumed: i64, buckets: &[BucketView]) -> (Vec<(i64, i64)>, i64) {
    if consumed <= 0 {
        return (Vec::new(), 0);
    }
    let mut order: Vec<&BucketView> = buckets.iter().filter(|b| b.remaining > 0).collect();
    order.sort_by_key(|b| (b.expires_at.is_none(), b.expires_at, b.remaining, b.id));

    let mut left = consumed;
    let mut takes = Vec::new();
    for bucket in order {
        if left == 0 {
            break;
        }
        let take = left.min(bucket.remaining);
        takes.push((bucket.id, take));
        left -= take;
    }
    (takes, left)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    fn bucket(id: i64, remaining: i64, expires_in_days: Option<i64>) -> BucketView {
        BucketView { id, remaining, expires_at: expires_in_days.map(|d| Utc::now() + Duration::days(d)) }
    }

    #[test]
    fn earliest_expiry_is_spent_first() {
        let buckets = [bucket(1, 100, Some(365)), bucket(2, 100, Some(7)), bucket(3, 100, Some(30))];
        assert_eq!(allocate(150, &buckets), (vec![(2, 100), (3, 50)], 0));
    }

    #[test]
    fn same_expiry_spends_the_smaller_bucket_first() {
        let at = Some(Utc::now() + Duration::days(7));
        let buckets = [
            BucketView { id: 1, remaining: 80, expires_at: at },
            BucketView { id: 2, remaining: 20, expires_at: at },
        ];
        assert_eq!(allocate(30, &buckets), (vec![(2, 20), (1, 10)], 0));
    }

    #[test]
    fn buckets_without_expiry_are_spent_last() {
        let buckets = [bucket(1, 50, None), bucket(2, 50, Some(400))];
        assert_eq!(allocate(60, &buckets), (vec![(2, 50), (1, 10)], 0));
    }

    #[test]
    fn overspend_is_reported_instead_of_dropped() {
        let buckets = [bucket(1, 40, Some(1))];
        assert_eq!(allocate(100, &buckets), (vec![(1, 40)], 60));
    }

    #[test]
    fn nothing_consumed_or_nothing_left_allocates_nothing() {
        let buckets = [bucket(1, 40, Some(1)), bucket(2, 0, Some(1))];
        assert_eq!(allocate(0, &buckets), (vec![], 0));
        assert_eq!(allocate(-5, &buckets), (vec![], 0));
        assert_eq!(allocate(10, &[]), (vec![], 10));
        assert_eq!(allocate(10, &[bucket(3, 0, None)]), (vec![], 10));
    }

    #[test]
    fn allocation_always_conserves_the_amount() {
        let buckets = [bucket(1, 7, Some(3)), bucket(2, 13, Some(1)), bucket(3, 29, None), bucket(4, 5, Some(1))];
        for consumed in 0..=70 {
            let (takes, overdraft) = allocate(consumed, &buckets);
            assert_eq!(takes.iter().map(|(_, t)| t).sum::<i64>() + overdraft, consumed);
            for (id, take) in takes {
                assert!(take <= buckets.iter().find(|b| b.id == id).unwrap().remaining);
            }
        }
    }
}
