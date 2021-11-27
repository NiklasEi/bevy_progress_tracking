//! A Bevy plugin to keep track of progress in your Application
//!
//! This little library can be used to track any kind of tasks. The most prominent example
//! would be asset loading, but you could also use it to keep track of necessary preparation steps
//! like world generation, or in-game tasks.

#![forbid(unsafe_code)]
#![warn(unused_imports, missing_docs)]

use bevy::app::{AppBuilder, Plugin};
use std::marker::PhantomData;

/// Bevy [Plugin] to keep track of any kind of progress in your application
pub struct ProgressTracker;

impl Plugin for ProgressTracker {
    fn build(&self, app: &mut AppBuilder) {
        app.init_resource::<Progress>();
    }
}

#[derive(Clone, Default, Debug, PartialEq)]
struct TaskProgress {
    tasks: usize,
    done: usize,
}

/// Resource that keeps record of current, previous and persisted progress
///
/// If you want to track different kinds of progress in parallel, you can use the
/// generic parameter to keep multiple progress resources.
#[derive(Default, Debug)]
pub struct Progress<T = ()> {
    current: TaskProgress,
    previous: TaskProgress,
    persisted: TaskProgress,

    _marker: PhantomData<T>,
}

/// Convenience enum to mark a single task as `in progress` or `done`
#[derive(PartialEq)]
pub enum Task {
    /// Mark a task as done
    Done,
    /// Mark a task as in progress
    InProgress,
}

impl TaskProgress {
    fn track(&mut self, tasks: usize, done: usize) {
        self.tasks += tasks;
        self.done += done;

        debug_assert!(self.tasks >= self.done, "The last track call adding {} tasks and {} done tasks led to more done tasks than there are tasks", tasks, done);
    }

    fn task(&mut self, task: Task) {
        if task == Task::Done {
            self.track(1, 1);
        } else {
            self.track(1, 0);
        }
    }

    fn clear(&mut self) {
        self.tasks = 0;
        self.done = 0;
    }
}

impl <T> Progress<T> {
    /// track the given amount of tasks of wich some can already be completed
    pub fn track(&mut self, tasks: usize, done: usize) {
        self.current.tasks += tasks;
        self.current.done += done;
    }

    /// Stop progress tracking for the given frame and clear the current count for the next frame
    ///
    /// This function should be called every frame before the progress is evaluated by calling [Progress::progress]
    pub fn finish_frame(&mut self) {
        self.track(self.persisted.tasks, self.persisted.done);
        self.previous = self.current.clone();
        self.current.clear();
    }

    /// Convenience function to track a single task
    ///
    /// ```edition2021
    /// # use bevy_progress_tracking::{Progress, Task};
    /// # let mut progress = Progress::default();
    /// progress.task(Task::Done);
    /// ```
    /// is the equivalent of `progress.track(1, 1)`.
    ///
    ///
    /// ```edition2021
    /// # use bevy_progress_tracking::{Progress, Task};
    /// # let mut progress = Progress::default();
    /// progress.task(Task::InProgress);
    /// ```
    /// is the equivalent of `progress.track(1, 0)`.
    pub fn task(&mut self, task: Task) {
        self.current.task(task);
    }

    /// Returns the progress as a floating point number between 0 and 1
    ///
    /// The values are taken from the last finished frame.
    /// You probably want to call [ProgressTracker::finish_frame] before calling this function.
    pub fn progress(&self) -> f32 {
        (self.previous.done as f32 / self.previous.tasks as f32).min(1.0)
    }

    /// Persist the given amount of tasks and mark them all as done
    ///
    /// Running the following once:
    /// ```edition2021
    /// # use bevy_progress_tracking::Progress;
    /// # let mut progress = Progress::default();
    /// progress.persist_done_tasks(42);
    /// ```
    /// is the equivalent of calling `progress.track(42, 42)` in *every* frame.
    pub fn persist_done_tasks(&mut self, done: usize) {
        self.persisted.track(done, done);
    }

    /// Persist the given amount of done tasks
    ///
    /// Running the following once:
    /// ```edition2021
    /// # use bevy_progress_tracking::Progress;
    /// # let mut progress = Progress::default();
    /// # progress.persist_tasks(42);
    /// progress.persist_done(42);
    /// ```
    /// is the equivalent of calling `progress.track(0, 42)` in every later *every* frame.
    pub fn persist_done(&mut self, done: usize) {
        self.persisted.track(0, done);
    }

    /// Persist the given amount of tasks
    ///
    /// Running the following once:
    /// ```edition2021
    /// # use bevy_progress_tracking::Progress;
    /// # let mut progress = Progress::default();
    /// progress.persist_tasks(42);
    /// ```
    /// is the equivalent of calling `progress.track(42, 0)` in every later *every* frame.
    pub fn persist_tasks(&mut self, tasks: usize) {
        self.persisted.track(tasks, 0);
    }

    /// Clear the progress resource
    ///
    /// This effectively resets all records
    pub fn clear(&mut self) {
        self.current.clear();
        self.previous.clear();
        self.persisted.clear();
    }
}

#[cfg(test)]
mod tests {
    use crate::{Progress, TaskProgress};

    #[test]
    fn correctly_tracks_persistent_tasks() {
        let mut progress = Progress::default();
        progress.persist_tasks(3);
        progress.persist_done(1);
        progress.persist_done_tasks(3);
        assert_eq!(progress.current, TaskProgress::default());
        assert_eq!(progress.previous, TaskProgress::default());

        progress.finish_frame();
        assert_eq!(progress.progress(), 4. / 6.);
        assert_eq!(progress.current, TaskProgress::default());
        assert_eq!(progress.previous, TaskProgress { tasks: 6, done: 4 });

        progress.finish_frame();
        assert_eq!(progress.progress(), 4. / 6.);
    }
}
