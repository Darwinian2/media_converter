pub struct Chapter {
    title: String,
    start_time: f64,
}

impl Chapter {
    #[allow(dead_code)]
    pub fn new(title: String, start_time: f64) -> Self {
        Chapter { title, start_time }
    }

    #[allow(dead_code)]
    pub fn create_chapters(chapter_data: Vec<(String, f64)>) -> Vec<Chapter> {
        chapter_data.into_iter().map(|(title, start_time)| Chapter::new(title, start_time)).collect()
    }

    #[allow(dead_code)]
    pub fn write_chapters(chapters: &[Chapter]) -> String {
        let mut chapter_metadata = String::new();
        for chapter in chapters {
            chapter_metadata.push_str(&format!("{} - {}\n", chapter.start_time, chapter.title));
        }
        chapter_metadata
    }
}