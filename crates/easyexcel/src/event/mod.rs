//! 对应 Java：`com.alibaba.excel.event.*`.

pub mod abstract_ignore_exception_read_listener;
pub mod analysis_event_listener;
pub mod handler;
pub mod listener;
pub mod not_repeat_executor;
pub mod order;
pub mod sync_read_listener;

pub use abstract_ignore_exception_read_listener::{
    AbstractIgnoreExceptionListenerAdapter, AbstractIgnoreExceptionReadListener,
};
pub use analysis_event_listener::{AnalysisEventListener, AnalysisEventListenerAdapter};
pub use handler::Handler;
pub use listener::Listener;
pub use not_repeat_executor::NotRepeatExecutor;
pub use order::Order;
pub use sync_read_listener::SyncReadListener;

pub use crate::context::analysis_context::{AnalysisContext, ErrorAction, Result};
pub mod page_read_listener;
pub use crate::read::listener::read_listener::{
    CompositeReadListener, ReadListener, ReadListenerList,
};
pub use page_read_listener::PageReadListener;
