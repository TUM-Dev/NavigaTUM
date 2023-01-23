use chrono::NaiveDate;

#[derive(Clone, Debug)]
pub(crate) struct ScrapeRoomTask {
    pub(crate) key: String,
    pub(crate) room_id: i32,
    pub(crate) from: NaiveDate,
    pub(crate) to: NaiveDate,
}

impl ScrapeRoomTask {
    pub fn new((key, room_id): (String, i32), from_year: i32, year_duration: i32) -> Self {
        let from = NaiveDate::from_ymd_opt(from_year, 1, 1).unwrap();
        let to = NaiveDate::from_ymd_opt(from_year + year_duration, 1, 1).unwrap()
            - chrono::Days::new(1);
        Self {
            key,
            room_id,
            from,
            to,
        }
    }
    pub fn num_days(&self) -> u64 {
        // we want to count from the morning of "from" to the evening of "to" => +1
        (self.to + chrono::Days::new(1))
            .signed_duration_since(self.from)
            .num_days() as u64
    }
    pub fn split(&self) -> (Self, Self) {
        let mid_offset = self.num_days() / 2 - 1;
        let lower_middle = self.from + chrono::Days::new(mid_offset);
        (
            Self {
                key: self.key.clone(),
                room_id: self.room_id,
                from: self.from,
                to: lower_middle,
            },
            Self {
                key: self.key.clone(),
                room_id: self.room_id,
                from: lower_middle + chrono::Days::new(1),
                to: self.to,
            },
        )
    }
}

#[cfg(test)]
mod test_scrape_task {
    use super::ScrapeRoomTask;
    use chrono::NaiveDate;
    #[test]
    fn test_split() {
        let task = ScrapeRoomTask::new(("".to_string(), 0), 2020, 1);
        let (o1, o2) = task.split();
        assert_eq!(task.from, NaiveDate::from_ymd_opt(2020, 1, 1).unwrap());
        assert_eq!(task.to, NaiveDate::from_ymd_opt(2020, 12, 31).unwrap());
        assert_eq!(o1.from, task.from);
        assert_eq!(o2.to, task.to);
        assert_eq!(o1.to + chrono::Duration::days(1), o2.from);
    }
    #[test]
    fn test_split_small() {
        let task = ScrapeRoomTask {
            key: "".to_string(),
            room_id: 0,
            from: NaiveDate::from_ymd_opt(2020, 1, 1).unwrap(),
            to: NaiveDate::from_ymd_opt(2020, 1, 2).unwrap(),
        };
        let (t1, t2) = task.split();
        assert_eq!(t1.to + chrono::Duration::days(1), t2.from);
        assert_eq!(task.num_days(), 2);
        assert_eq!(task.from, t1.from);
        assert_eq!(task.to, t2.to);
        assert_eq!(t1.num_days(), 1);
        assert_eq!(t2.num_days(), 1);
        assert_eq!(task.from, t1.to);
        assert_eq!(task.to, t2.from);
    }
    #[test]
    fn test_num_days() {
        let mut task = ScrapeRoomTask {
            key: "".to_string(),
            room_id: 0,
            from: NaiveDate::from_ymd_opt(2020, 1, 1).unwrap(),
            to: NaiveDate::from_ymd_opt(2020, 1, 1).unwrap(),
        };
        assert_eq!(task.num_days(), 1);
        task.to = NaiveDate::from_ymd_opt(2020, 1, 2).unwrap();
        assert_eq!(task.num_days(), 2);
        task.to = NaiveDate::from_ymd_opt(2020, 12, 31).unwrap();
        assert_eq!(task.num_days(), 366);
    }
    #[test]
    fn test_same_day() {
        let task = ScrapeRoomTask::new(("".to_string(), 0), 2020, 0);
        assert_eq!(task.from, NaiveDate::from_ymd_opt(2020, 1, 1).unwrap());
        assert_eq!(task.to, NaiveDate::from_ymd_opt(2019, 12, 31).unwrap());
        assert_eq!(task.num_days(), 0);
    }
}
