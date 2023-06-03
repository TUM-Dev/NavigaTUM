use crate::scrape_task::main_api_connector::Room;
use chrono::NaiveDate;

#[derive(Clone, Debug)]
pub(crate) struct ScrapeRoomTask {
    pub(crate) room: Room,
    pub(crate) from: NaiveDate,
    pub(crate) to: NaiveDate,
}

impl ScrapeRoomTask {
    pub fn new(room: Room, from: NaiveDate, duration: chrono::Duration) -> Self {
        let to = from + duration;
        Self { room, from, to }
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
                room: self.room.clone(),
                from: self.from,
                to: lower_middle,
            },
            Self {
                room: self.room.clone(),
                from: lower_middle + chrono::Days::new(1),
                to: self.to,
            },
        )
    }
}

#[cfg(test)]
mod test_scrape_task {
    use super::ScrapeRoomTask;
    use crate::scrape_task::main_api_connector::Room;
    use chrono::NaiveDate;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_split() {
        let start = NaiveDate::from_ymd_opt(2020, 1, 1).unwrap();
        let task = ScrapeRoomTask::new(Room::default(), start, chrono::Duration::days(365));
        let (o1, o2) = task.split();
        assert_eq!(task.from, start);
        assert_eq!(task.to, NaiveDate::from_ymd_opt(2020, 12, 31).unwrap());
        assert_eq!(o1.from, task.from);
        assert_eq!(o2.to, task.to);
        assert_eq!(o1.to + chrono::Duration::days(1), o2.from);
    }
    #[test]
    fn test_split_small() {
        let task = ScrapeRoomTask {
            room: Room::default(),
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
            room: Room::default(),
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
        let start = NaiveDate::from_ymd_opt(2020, 1, 1).unwrap();
        let task = ScrapeRoomTask::new(Room::default(), start, chrono::Duration::days(0));
        assert_eq!(task.from, start);
        assert_eq!(task.to, NaiveDate::from_ymd_opt(2020, 1, 1).unwrap());
        assert_eq!(task.num_days(), 1);
    }
}
