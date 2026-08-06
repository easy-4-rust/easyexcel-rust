//! BIFF SID-to-handler dispatch matching Java `XlsSaxAnalyser.processRecord`.

use crate::core::{CellExtraType, Result};
use easyexcel_xls::biff8::{
    Biff8ContinuableRecordDecoder, Biff8ContinuableRecordKind, Biff8ContinuationStatus,
    Biff8DecodedContinuableRecord,
};

use crate::{ReadOptions, SheetSelector};

use super::handlers::blank_record_handler::{BLANK_SID, BlankCell, BlankRecordHandler};
use super::handlers::bof_record_handler::{BOF_SID, BofRecordHandler};
use super::handlers::bool_err_record_handler::{BOOL_ERR_SID, BoolCell, BoolErrRecordHandler};
use super::handlers::bound_sheet_record_handler::{
    BOUND_SHEET_SID, BoundSheetEntry, BoundSheetRecordHandler,
};
use super::handlers::dummy_record_handler::DummyRecordHandler;
use super::handlers::eof_record_handler::{EOF_SID, EofRecordHandler};
use super::handlers::formula_record_handler::{FORMULA_SID, FormulaCell, FormulaRecordHandler};
use super::handlers::hyperlink_record_handler::HyperlinkRecordHandler;
use super::handlers::index_record_handler::{INDEX_SID, IndexRecordHandler};
use super::handlers::label_record_handler::{LABEL_SID, LabelRecordHandler};
use super::handlers::label_sst_record_handler::{
    LABEL_SST_SID, LabelSstCell, LabelSstRecordHandler,
};
use super::handlers::merge_cells_record_handler::MergeCellsRecordHandler;
use super::handlers::note_record_handler::NoteRecordHandler;
use super::handlers::number_record_handler::{NUMBER_SID, NumberCell, NumberRecordHandler};
use super::handlers::obj_record_handler::{OBJ_SID, ObjRecordHandler};
use super::handlers::rk_record_handler::{RK_SID, RkRecordHandler};
use super::handlers::sst_record_handler::{SST_SID, SstRecordHandler};
use super::handlers::string_record_handler::{STRING_SID, StringRecordHandler};
use super::handlers::text_object_record_handler::{TEXT_OBJECT_SID, TextObjectRecordHandler};
use super::xls_record_handler::XlsRecordHandler;

const HYPERLINK_SID: u16 = easyexcel_xls::biff8::record_sid::HYPERLINK_SID;
const MERGE_CELLS_SID: u16 = easyexcel_xls::biff8::record_sid::MERGE_CELLS_SID;
const NOTE_SID: u16 = easyexcel_xls::biff8::record_sid::NOTE_SID;
const DUMMY_RECORD_SID: u16 = u16::MAX;
const CONTINUE_SID: u16 = easyexcel_xls::biff8::record_sid::CONTINUE_SID;

include!("xls_record_dispatcher/xls_record_dispatch_state.rs");

/// 对应 Java：XlsSaxAnalyser.processRecord。 Owns the 19 Java `EasyExcel` XLS handlers and dispatches by BIFF SID.
#[derive(Debug)]
pub struct XlsRecordDispatcher {
    state: XlsRecordDispatchState,
    blank: BlankRecordHandler,
    bof: BofRecordHandler,
    bool_err: BoolErrRecordHandler,
    bound_sheet: BoundSheetRecordHandler,
    dummy: DummyRecordHandler,
    eof: EofRecordHandler,
    formula: FormulaRecordHandler,
    hyperlink: HyperlinkRecordHandler,
    index: IndexRecordHandler,
    label: LabelRecordHandler,
    label_sst: LabelSstRecordHandler,
    merge_cells: MergeCellsRecordHandler,
    note: NoteRecordHandler,
    number: NumberRecordHandler,
    obj: ObjRecordHandler,
    rk: RkRecordHandler,
    sst: SstRecordHandler,
    string: StringRecordHandler,
    text_object: TextObjectRecordHandler,
    sheet_selector: SheetSelector,
    next_sheet_index: usize,
    ignore_record: bool,
    auto_trim: bool,
    continuable_record: Biff8ContinuableRecordDecoder,
}

impl XlsRecordDispatcher {
    /// 对应 Java：XlsSaxAnalyser.processRecord。 Creates the handler map using Java `support(context)` feature flags.
    #[must_use]
    pub fn new(options: &ReadOptions) -> Self {
        Self {
            state: XlsRecordDispatchState::default(),
            blank: BlankRecordHandler::new(),
            bof: BofRecordHandler::new(),
            bool_err: BoolErrRecordHandler::new(),
            bound_sheet: BoundSheetRecordHandler::new(),
            dummy: DummyRecordHandler::new(),
            eof: EofRecordHandler::new(),
            formula: FormulaRecordHandler::new(),
            hyperlink: HyperlinkRecordHandler::new(
                options.extra_read.contains(&CellExtraType::Hyperlink),
            ),
            index: IndexRecordHandler::new(),
            label: LabelRecordHandler::new(),
            label_sst: LabelSstRecordHandler::new(),
            merge_cells: MergeCellsRecordHandler::new(
                options.extra_read.contains(&CellExtraType::Merge),
            ),
            note: NoteRecordHandler::new(options.extra_read.contains(&CellExtraType::Comment)),
            number: NumberRecordHandler::new(),
            obj: ObjRecordHandler::new(),
            rk: RkRecordHandler::new(),
            sst: SstRecordHandler::new(),
            string: StringRecordHandler::new(),
            text_object: TextObjectRecordHandler::new(),
            sheet_selector: options.sheet.clone(),
            next_sheet_index: 0,
            ignore_record: false,
            auto_trim: options.auto_trim,
            continuable_record: Biff8ContinuableRecordDecoder::default(),
        }
    }

    /// 对应 Java：XlsSaxAnalyser.processRecord。 Resets all per-workbook state while preserving configured feature flags.
    pub fn reset(&mut self) {
        let hyperlink_enabled = self.hyperlink.enabled;
        let merge_enabled = self.merge_cells.enabled;
        let note_enabled = self.note.enabled;
        let sheet_selector = self.sheet_selector.clone();
        let auto_trim = self.auto_trim;
        *self = Self {
            state: XlsRecordDispatchState::default(),
            blank: BlankRecordHandler::new(),
            bof: BofRecordHandler::new(),
            bool_err: BoolErrRecordHandler::new(),
            bound_sheet: BoundSheetRecordHandler::new(),
            dummy: DummyRecordHandler::new(),
            eof: EofRecordHandler::new(),
            formula: FormulaRecordHandler::new(),
            hyperlink: HyperlinkRecordHandler::new(hyperlink_enabled),
            index: IndexRecordHandler::new(),
            label: LabelRecordHandler::new(),
            label_sst: LabelSstRecordHandler::new(),
            merge_cells: MergeCellsRecordHandler::new(merge_enabled),
            note: NoteRecordHandler::new(note_enabled),
            number: NumberRecordHandler::new(),
            obj: ObjRecordHandler::new(),
            rk: RkRecordHandler::new(),
            sst: SstRecordHandler::new(),
            string: StringRecordHandler::new(),
            text_object: TextObjectRecordHandler::new(),
            sheet_selector,
            next_sheet_index: 0,
            ignore_record: false,
            auto_trim,
            continuable_record: Biff8ContinuableRecordDecoder::default(),
        };
    }

    /// Returns observable dispatch state for diagnostics and parity tests.
    #[must_use]
    /// 对应 Java：XlsSaxAnalyser.processRecord。
    pub const fn state(&self) -> &XlsRecordDispatchState {
        &self.state
    }

    /// 对应 Java：XlsSaxAnalyser.processRecord。 对应 Java：`XlsSaxAnalyser.processRecord`: unknown SIDs are ignored,
    /// disabled handlers are skipped, and known records reach their handler.
    ///
    /// # Errors
    ///
    /// 当待收尾的 SST/公式字符串记录解码失败时返回 `ExcelError`。
    // 对应 Java：`processRecord` 的大 switch 与 POI handler 路由顺序一一对应；
    // SST/STRING 的物理 CONTINUE 生命周期由 easyexcel-xls 状态机管理。
    #[allow(clippy::too_many_lines)]
    /// 对应 Java：XlsSaxAnalyser.processRecord。
    pub fn process_record(&mut self, record_sid: u16, data: &[u8]) -> Result<()> {
        self.state.total_record_count += 1;
        if record_sid == CONTINUE_SID {
            if self.continuable_record.push(data) {
                self.try_finalize_continuable_record(false)?;
                return Ok(());
            }
            self.state.unknown_record_count += 1;
            return Ok(());
        }
        self.finish_pending_records()?;
        if self.ignore_record
            && (record_sid == DUMMY_RECORD_SID
                || easyexcel_xls::biff8::record_sid::is_skippable_event_record(record_sid))
        {
            self.state.skipped_record_count += 1;
            return Ok(());
        }
        match record_sid {
            BLANK_SID => self.dispatch_blank(record_sid, data),
            BOF_SID => {
                if let Some(bof_type) = easyexcel_xls::biff8::event_record::decode_bof_type(data) {
                    match bof_type {
                        easyexcel_xls::biff8::event_record::Biff8BofType::Workbook => {
                            self.state.workbook_bof_count += 1;
                            self.next_sheet_index = 0;
                            self.ignore_record = false;
                        }
                        easyexcel_xls::biff8::event_record::Biff8BofType::Worksheet => {
                            self.state.worksheet_bof_count += 1;
                            self.ignore_record = !self.should_read_sheet(self.next_sheet_index);
                            self.next_sheet_index = self.next_sheet_index.saturating_add(1);
                        }
                        easyexcel_xls::biff8::event_record::Biff8BofType::Other(_) => {}
                    }
                }
                self.bof.process_record(record_sid, data);
            }
            BOOL_ERR_SID => self.dispatch_bool(record_sid, data),
            BOUND_SHEET_SID => {
                self.bound_sheet.process_record(record_sid, data);
                self.state.bound_sheets = self.bound_sheet.ordered_sheets();
            }
            DUMMY_RECORD_SID => self.dummy.process_record(record_sid, data),
            EOF_SID => {
                self.state.eof_count += 1;
                self.eof.process_record(record_sid, data);
            }
            FORMULA_SID => {
                self.formula.process_record(record_sid, data);
                self.state.last_formula_cell = self.formula.last_cell.clone();
            }
            HYPERLINK_SID => {
                if !self.hyperlink.support() {
                    self.state.skipped_record_count += 1;
                    return Ok(());
                }
                self.hyperlink.process_record(record_sid, data);
            }
            INDEX_SID => {
                self.index.process_record(record_sid, data);
                self.state.approximate_total_row_number = self.index.approximate_total_row_number;
            }
            LABEL_SID => self.label.process_record(record_sid, data),
            LABEL_SST_SID => {
                self.label_sst.process_record(record_sid, data);
                if let Some(reference) = self.label_sst.last_reference {
                    let cell = LabelSstRecordHandler::process_label_sst(
                        reference.row,
                        reference.column,
                        reference.sst_index,
                        self.auto_trim,
                        &|index| self.sst.get(index).map(str::to_owned),
                    );
                    self.state.last_label_sst_cell = Some(cell);
                }
            }
            MERGE_CELLS_SID => {
                if !self.merge_cells.support() {
                    self.state.skipped_record_count += 1;
                    return Ok(());
                }
                self.merge_cells.process_record(record_sid, data);
            }
            NOTE_SID => {
                if !self.note.support() {
                    self.state.skipped_record_count += 1;
                    return Ok(());
                }
                self.note.process_record(record_sid, data);
            }
            NUMBER_SID => self.dispatch_number(record_sid, data),
            OBJ_SID => self.obj.process_record(record_sid, data),
            RK_SID => self.dispatch_rk(record_sid, data),
            SST_SID => {
                self.sst.process_record(record_sid, data);
                self.state.unique_string_count = self.sst.unique_string_count;
                self.continuable_record
                    .begin(Biff8ContinuableRecordKind::SharedStringTable, data);
                self.try_finalize_continuable_record(false)?;
            }
            STRING_SID => {
                self.continuable_record
                    .begin(Biff8ContinuableRecordKind::UnicodeString, data);
                self.try_finalize_continuable_record(false)?;
            }
            TEXT_OBJECT_SID => self.text_object.process_record(record_sid, data),
            _ => {
                self.state.unknown_record_count += 1;
                return Ok(());
            }
        }
        self.state.handled_record_count += 1;
        Ok(())
    }

    /// 对应 Java：XlsSaxAnalyser.processRecord。 Finalizes a continuable logical record at end-of-stream.
    ///
    /// # Errors
    ///
    /// 当待收尾的 SST/公式字符串记录解码失败时返回 `ExcelError`。
    pub fn finish_records(&mut self) -> Result<()> {
        self.finish_pending_records()
    }

    fn dispatch_blank(&mut self, record_sid: u16, data: &[u8]) {
        self.blank.process_record(record_sid, data);
        self.state.last_blank_cell = self.blank.last_cell;
    }

    fn dispatch_bool(&mut self, record_sid: u16, data: &[u8]) {
        self.bool_err.process_record(record_sid, data);
        self.state.last_boolean_cell = self.bool_err.last_cell;
    }

    fn dispatch_number(&mut self, record_sid: u16, data: &[u8]) {
        self.number.process_record(record_sid, data);
        self.state.last_number_cell = self.number.last_cell.clone();
    }

    fn dispatch_rk(&mut self, record_sid: u16, data: &[u8]) {
        self.rk.process_record(record_sid, data);
        self.state.last_rk_cell = self.rk.last_cell;
    }

    fn should_read_sheet(&self, index: usize) -> bool {
        self.sheet_selector.as_engine_selection().matches(
            index,
            self.state
                .bound_sheets
                .get(index)
                .map(|sheet| sheet.name.as_str()),
            self.auto_trim,
        )
    }

    fn finish_pending_records(&mut self) -> Result<()> {
        self.try_finalize_continuable_record(true)
    }

    fn try_finalize_continuable_record(&mut self, require_complete: bool) -> Result<()> {
        match self.continuable_record.try_finish(require_complete)? {
            Biff8ContinuationStatus::Complete(Biff8DecodedContinuableRecord::SharedStrings(
                strings,
            )) => {
                let unique = u32::try_from(strings.len()).map_err(|_| {
                    crate::core::ExcelError::Format(
                        "decoded SST size exceeds BIFF u32 range".to_owned(),
                    )
                })?;
                self.sst.process_decoded_sst(unique, strings.clone());
                self.state.unique_string_count = Some(unique);
                self.state.shared_strings = strings;
            }
            Biff8ContinuationStatus::Complete(Biff8DecodedContinuableRecord::UnicodeString(
                value,
            )) => {
                self.string.process_decoded(value.clone());
                if let Some((cell, _)) =
                    StringRecordHandler::process_string(&mut self.formula, value, self.auto_trim)
                {
                    self.state.last_formula_cell = Some(cell);
                }
            }
            Biff8ContinuationStatus::Idle | Biff8ContinuationStatus::Pending => {}
        }
        Ok(())
    }
}

impl Default for XlsRecordDispatcher {
    fn default() -> Self {
        Self::new(&ReadOptions::default())
    }
}

#[cfg(test)]
#[path = "xls_record_dispatcher_tests/tests.rs"]
mod tests;

#[cfg(test)]
#[path = "xls_record_dispatcher_tests/tests_extra.rs"]
mod tests_extra;

#[cfg(test)]
#[path = "xls_record_dispatcher_tests/tests_extra2.rs"]
mod tests_extra2;
