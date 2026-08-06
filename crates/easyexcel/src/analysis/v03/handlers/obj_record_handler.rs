//! 对应 Java：`com.alibaba.excel.analysis.v03.handlers.ObjRecordHandler`.
//!
//! Tracks the current drawing/object id used later by note/text handlers.

use super::super::xls_record_handler::XlsRecordHandler;

/// 对应 Java：`ObjRecordHandler`.
#[derive(Debug, Default)]
pub struct ObjRecordHandler {
    /// Last seen object / shape id. (Java sheet holder)
    pub temp_object_index: Option<u32>,
}

impl ObjRecordHandler {
    /// Creates an idle handler.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Java `ObjRecordHandler.processRecord`.
    pub fn process_obj(&mut self, object_id: u32) {
        self.temp_object_index = Some(object_id);
    }
}

/// BIFF `Obj` record sid. (POI `ObjRecord.sid`)
pub use easyexcel_xls::biff8::record_sid::OBJ_SID;

impl XlsRecordHandler for ObjRecordHandler {
    /// Java `ObjRecordHandler.processRecord` — sid gate; object id via [`Self::process_obj`].
    fn process_record(&mut self, record_sid: u16, data: &[u8]) {
        if record_sid != OBJ_SID {
            return;
        }
        if let Some(common) = easyexcel_xls::biff8::event_record::decode_obj_common_data(data)
            && common.object_type
                == easyexcel_xls::biff8::event_record::BIFF8_OBJECT_TYPE_COMMENT
        {
            self.process_obj(common.object_id);
        }
    }
}

#[cfg(test)]
mod tests_extra {
    use super::*;

    #[test]
    fn process_obj_tracks_object_id() {
        // 对应 Java：ObjRecordHandler 记录当前对象 id
        let mut handler = ObjRecordHandler::new();
        handler.process_obj(7);
        assert_eq!(handler.temp_object_index, Some(7));
        // sid 门控：仅 OBJ_SID 触发
        handler.process_record(OBJ_SID, &[]);
        handler.process_record(0xFFFF, &[]);
        assert_eq!(handler.temp_object_index, Some(7));
    }
}
