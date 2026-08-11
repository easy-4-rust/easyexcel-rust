//! 可复用的 OOXML 模板包修改引擎。
//!
//! 本模块负责工作表部件定位、创建、行追加、布局修改和样式表合并，
//! 不依赖 `EasyExcel` builder、listener、annotation 或门面值类型。

use std::io::Write;
use std::ops::{Deref, DerefMut};
use std::path::Path;

use easyexcel_io::{Error, Result};
use zip::CompressionMethod;

use super::ooxml_package::{OoxmlPackage, OoxmlZipEntry};
use super::package::{relationship_part_name, resolve_target};
use super::template_fill::{
    TemplateComment, TemplateHyperlink, TemplateHyperlinkType, TemplateImage, TemplateImageMovement,
};
use super::template_styles::{merge_compiled_styles, merge_compiled_styles_onto};
use super::template_xml::{
    TemplateCellValue, TemplateMergeRange, append_sparse_rows, apply_column_widths,
    apply_merge_ranges, apply_sheet_protection, attribute_value, cell_style_index, escape_xml,
    set_cell_value, worksheet_max_row,
};

const WORKBOOK_PATH: &str = "xl/workbook.xml";
const WORKBOOK_RELS_PATH: &str = "xl/_rels/workbook.xml.rels";
const CONTENT_TYPES_PATH: &str = "[Content_Types].xml";
const STYLES_PATH: &str = "xl/styles.xml";

const EMPTY_WORKSHEET_XML: &str = concat!(
    r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>"#,
    r#"<worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main" "#,
    r#"xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">"#,
    r#"<dimension ref="A1"/><sheetData></sheetData></worksheet>"#
);

/// 对应 Java：无直接对应对象；Rust 架构扩展。 保留未知部件的 OOXML 模板工作簿。
#[derive(Debug, Clone)]
pub struct OoxmlTemplatePackage {
    entries: OoxmlPackage,
}

impl OoxmlTemplatePackage {
    /// 对应 Java：无直接对应对象；Rust 架构扩展。 从 XLSX/OOXML 字节载入模板包。
    ///
    /// # Errors
    ///
    /// 底层 OOXML、ZIP、XML 或目标 I/O 操作失败，或输入不符合格式约束时返回错误。
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        Ok(Self {
            entries: OoxmlPackage::from_bytes(bytes)?,
        })
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 使用已载入的 OOXML 条目包构建模板引擎。
    #[must_use]
    pub fn from_package(entries: OoxmlPackage) -> Self {
        Self { entries }
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 取出底层无损 OOXML 包。
    #[must_use]
    pub fn into_package(self) -> OoxmlPackage {
        self.entries
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 按工作簿顺序返回工作表名称。
    ///
    /// # Errors
    ///
    /// 底层 OOXML、ZIP、XML 或目标 I/O 操作失败，或输入不符合格式约束时返回错误。
    pub fn sheet_names(&self) -> Result<Vec<String>> {
        Ok(self
            .workbook_sheets()?
            .into_iter()
            .map(|(name, _)| name)
            .collect())
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 返回指定工作表下一条可追加的零基行号。
    ///
    /// # Errors
    ///
    /// 底层 OOXML、ZIP、XML 或目标 I/O 操作失败，或输入不符合格式约束时返回错误。
    pub fn next_row_for_sheet(&self, sheet_name: &str) -> Result<u32> {
        let path = self.worksheet_path_by_name(sheet_name)?;
        let xml = self.entry_xml(&path)?;
        let maximum = worksheet_max_row(&xml);
        if maximum == 0 && !xml.contains("<row") {
            Ok(0)
        } else {
            Ok(u32::try_from(maximum.saturating_add(1)).unwrap_or(u32::MAX))
        }
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 按工作表名称解析对应的 worksheet part 路径。
    ///
    /// # Errors
    ///
    /// 底层 OOXML、ZIP、XML 或目标 I/O 操作失败，或输入不符合格式约束时返回错误。
    pub fn worksheet_path_by_name(&self, sheet_name: &str) -> Result<String> {
        let sheets = self.workbook_sheets()?;
        let selected = sheets
            .iter()
            .find(|(name, _)| name == sheet_name)
            .ok_or_else(|| Error::SheetNotFound(sheet_name.to_owned()))?;
        self.worksheet_part_for_relationship(&selected.1, &selected.0)
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 按零基工作表下标返回名称与 worksheet part 路径。
    ///
    /// # Errors
    ///
    /// 底层 OOXML、ZIP、XML 或目标 I/O 操作失败，或输入不符合格式约束时返回错误。
    pub fn worksheet_path_by_index(&self, index: usize) -> Result<(String, String)> {
        let sheets = self.workbook_sheets()?;
        let selected = sheets
            .get(index)
            .ok_or_else(|| Error::SheetNotFound(index.to_string()))?;
        let path = self.worksheet_part_for_relationship(&selected.1, &selected.0)?;
        Ok((selected.0.clone(), path))
    }

    /// 按 worksheet part 路径反向解析工作表名称。
    pub fn sheet_name_by_worksheet_path(&self, worksheet_path: &str) -> Result<String> {
        for (sheet_name, relationship_id) in self.workbook_sheets()? {
            let path = self.worksheet_part_for_relationship(&relationship_id, &sheet_name)?;
            if path.eq_ignore_ascii_case(worksheet_path) {
                return Ok(sheet_name);
            }
        }
        Err(Error::SheetNotFound(worksheet_path.to_owned()))
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 确保指定名称的工作表存在。
    ///
    /// # Errors
    ///
    /// 底层 OOXML、ZIP、XML 或目标 I/O 操作失败，或输入不符合格式约束时返回错误。
    pub fn ensure_sheet(&mut self, sheet_name: &str) -> Result<()> {
        if self.sheet_names()?.iter().any(|name| name == sheet_name) {
            return Ok(());
        }
        self.create_sheet(sheet_name)
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 创建空工作表并继承第一个工作表的默认行高和列宽。
    ///
    /// # Errors
    ///
    /// 底层 OOXML、ZIP、XML 或目标 I/O 操作失败，或输入不符合格式约束时返回错误。
    pub fn create_sheet(&mut self, sheet_name: &str) -> Result<()> {
        let sheet_part = next_worksheet_part_name(&self.entries);
        let workbook_index = entry_index(&self.entries, WORKBOOK_PATH)?;
        let rels_index = entry_index(&self.entries, WORKBOOK_RELS_PATH)?;
        let content_types_index = entry_index(&self.entries, CONTENT_TYPES_PATH)?;

        let workbook_xml = entry_string(&self.entries[workbook_index])?;
        let rels_xml = entry_string(&self.entries[rels_index])?;
        let content_types_xml = entry_string(&self.entries[content_types_index])?;
        let relationship_id = next_relationship_id(&rels_xml);
        let sheet_id = next_sheet_id(&workbook_xml);
        let escaped_name = escape_xml(sheet_name);
        let sheet_tag = format!(
            "<sheet name=\"{escaped_name}\" sheetId=\"{sheet_id}\" r:id=\"{relationship_id}\"/>"
        );
        let relationship_tag = format!(
            "<Relationship Id=\"{relationship_id}\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/worksheet\" Target=\"worksheets/{}\"/>",
            sheet_part
                .strip_prefix("xl/worksheets/")
                .unwrap_or(sheet_part.as_str())
        );
        let override_tag = format!(
            "<Override PartName=\"/{sheet_part}\" ContentType=\"application/vnd.openxmlformats-officedocument.spreadsheetml.worksheet+xml\"/>"
        );

        self.entries[workbook_index].bytes =
            insert_before_close_tag(&workbook_xml, "</sheets>", &sheet_tag)?.into_bytes();
        self.entries[rels_index].bytes =
            insert_before_close_tag(&rels_xml, "</Relationships>", &relationship_tag)?.into_bytes();
        self.entries[content_types_index].bytes =
            insert_before_close_tag(&content_types_xml, "</Types>", &override_tag)?.into_bytes();
        let bytes = blank_worksheet_with_inherited_format(&self.entries);
        self.entries.push(OoxmlZipEntry {
            name: sheet_part,
            is_dir: false,
            compression: CompressionMethod::Deflated,
            unix_mode: None,
            bytes,
        });
        Ok(())
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 追加稀疏行、行高和样式索引，并保留显式缺席的 Java `null` 行。
    ///
    /// # Errors
    ///
    /// 底层 OOXML、ZIP、XML 或目标 I/O 操作失败，或输入不符合格式约束时返回错误。
    pub fn append_rows(
        &mut self,
        sheet_name: &str,
        rows: &[Vec<(usize, TemplateCellValue)>],
        row_heights: &[Option<u16>],
        cell_styles: &[Vec<Option<u32>>],
        absent_rows: &[bool],
    ) -> Result<u32> {
        if rows.is_empty() {
            return self.next_row_for_sheet(sheet_name);
        }
        validate_row_shapes(rows, row_heights, cell_styles, absent_rows)?;
        let path = self.worksheet_path_by_name(sheet_name)?;
        let entry = self.entry_mut(&path)?;
        let xml = String::from_utf8(std::mem::take(&mut entry.bytes))
            .map_err(|error| Error::Xlsx(error.to_string()))?;
        let (updated, next_row) =
            append_sparse_rows(&xml, rows, row_heights, cell_styles, absent_rows)?;
        entry.bytes = updated.into_bytes();
        Ok(next_row)
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 应用列宽和绝对合并区域。
    ///
    /// # Errors
    ///
    /// 底层 OOXML、ZIP、XML 或目标 I/O 操作失败，或输入不符合格式约束时返回错误。
    pub fn apply_sheet_layout(
        &mut self,
        sheet_name: &str,
        column_widths: &[(u16, u16)],
        merge_ranges: &[TemplateMergeRange],
    ) -> Result<()> {
        if column_widths.is_empty() && merge_ranges.is_empty() {
            return Ok(());
        }
        let path = self.worksheet_path_by_name(sheet_name)?;
        let entry = self.entry_mut(&path)?;
        let mut xml = String::from_utf8(std::mem::take(&mut entry.bytes))
            .map_err(|error| Error::Xlsx(error.to_string()))?;
        if !column_widths.is_empty() {
            xml = apply_column_widths(&xml, column_widths)?;
        }
        if !merge_ranges.is_empty() {
            xml = apply_merge_ranges(&xml, merge_ranges)?;
        }
        entry.bytes = xml.into_bytes();
        Ok(())
    }

    /// 在模板工作表中新增或替换一个单元格值。
    ///
    /// # Errors
    ///
    /// 工作表不存在、坐标无效或工作表 XML 损坏时返回错误。
    pub fn set_cell(
        &mut self,
        sheet_name: &str,
        row_index: u32,
        column_index: u16,
        value: &TemplateCellValue,
    ) -> Result<()> {
        let path = self.worksheet_path_by_name(sheet_name)?;
        let entry = self.entry_mut(&path)?;
        let xml = String::from_utf8(std::mem::take(&mut entry.bytes))
            .map_err(|error| Error::Xlsx(error.to_string()))?;
        entry.bytes = set_cell_value(&xml, row_index, column_index, value)?.into_bytes();
        Ok(())
    }

    /// 删除指定单元格的传统 OOXML 批注，并同步移除对应 VML shape。
    ///
    /// 对应 Java：`XSSFCell#removeCellComment()`。comments、worksheet
    /// relationship 与 VML 均由 XLSX 引擎处理，调用层不接触 XML。
    ///
    /// # Errors
    ///
    /// 工作表关系、comments XML 或 VML XML 损坏时返回错误；工作表没有该
    /// 批注时返回 `Ok(false)`。
    pub fn remove_comment(
        &mut self,
        sheet_name: &str,
        row_index: u32,
        column_index: u16,
    ) -> Result<bool> {
        let sheet_path = self.worksheet_path_by_name(sheet_name)?;
        let relationships_path = relationship_part_name(&sheet_path);
        let relationships_xml = match self.entry_xml(&relationships_path) {
            Ok(xml) => xml,
            Err(_) => return Ok(false),
        };
        let Some(comments_target) = relationship_target_by_type(&relationships_xml, "/comments")
        else {
            return Ok(false);
        };
        let comments_path = resolve_target(&sheet_path, &comments_target)?;
        let comments_xml = self.entry_xml(&comments_path)?;
        let reference = a1_reference(row_index, column_index);
        let (comments_xml, removed) =
            remove_xml_element_by_attribute(&comments_xml, "comment", "ref", &reference)?;
        if !removed {
            return Ok(false);
        }
        self.entry_mut(&comments_path)?.bytes = comments_xml.into_bytes();

        if let Some(vml_target) = relationship_target_by_type(&relationships_xml, "/vmlDrawing") {
            let vml_path = resolve_target(&sheet_path, &vml_target)?;
            let vml_xml = self.entry_xml(&vml_path)?;
            let (vml_xml, _) = remove_vml_comment_shape(
                &vml_xml,
                usize::try_from(row_index).unwrap_or(usize::MAX),
                usize::from(column_index),
            )?;
            self.entry_mut(&vml_path)?.bytes = vml_xml.into_bytes();
        }
        Ok(true)
    }

    /// 将编译工作簿中的单个传统批注移植到模板工作表。
    ///
    /// comments、VML、worksheet relationships 与 content types 均在引擎层
    /// 合并；调用方只负责用统一 XLSX 生成器编译批注语义。
    pub fn import_comment(
        &mut self,
        compiled_xlsx: &[u8],
        sheet_name: &str,
        row_index: u32,
        column_index: u16,
    ) -> Result<()> {
        let snapshot = self.clone();
        if let Err(error) =
            self.import_comment_inner(compiled_xlsx, sheet_name, row_index, column_index)
        {
            *self = snapshot;
            return Err(error);
        }
        Ok(())
    }

    /// 按引擎中立语义在模板工作表中新增或覆盖传统批注。
    pub fn set_template_comment(
        &mut self,
        sheet_name: &str,
        row_index: u32,
        column_index: u16,
        comment: &TemplateComment,
    ) -> Result<()> {
        let mut compiler = super::generation::new_workbook();
        let worksheet = super::generation::create_worksheet(&mut compiler, sheet_name, false)?;
        let movement = comment.movement.and_then(|movement| match movement {
            0 => Some(super::generation::ObjectMovement::MoveAndSizeWithCells),
            1 => Some(super::generation::ObjectMovement::MoveButDontSizeWithCells),
            2 => Some(super::generation::ObjectMovement::DontMoveOrSizeWithCells),
            _ => None,
        });
        super::generation::insert_note_with_metadata(
            worksheet,
            row_index,
            column_index,
            &comment.text,
            comment.author.as_deref(),
            movement,
            comment.visible,
        )?;
        let bytes = super::generation::serialize_workbook(&mut compiler)?;
        self.import_comment(&bytes, sheet_name, row_index, column_index)
    }

    /// 按引擎中立语义在模板工作表中新增或覆盖超链接。
    ///
    /// 工作簿内部链接使用 worksheet `location`；URL、邮件和文件链接使用外部
    /// relationship。修改在快照上事务提交，关系或 XML 失败时恢复原模板包。
    pub fn set_template_hyperlink(
        &mut self,
        sheet_name: &str,
        row_index: u32,
        column_index: u16,
        hyperlink: &TemplateHyperlink,
    ) -> Result<()> {
        let snapshot = self.clone();
        if let Err(error) =
            self.set_template_hyperlink_inner(sheet_name, row_index, column_index, hyperlink)
        {
            *self = snapshot;
            return Err(error);
        }
        Ok(())
    }

    fn set_template_hyperlink_inner(
        &mut self,
        sheet_name: &str,
        row_index: u32,
        column_index: u16,
        hyperlink: &TemplateHyperlink,
    ) -> Result<()> {
        let sheet_path = self.worksheet_path_by_name(sheet_name)?;
        let relationships_path = relationship_part_name(&sheet_path);
        let reference = hyperlink.resolve_reference(row_index, column_index)?;
        let mut sheet_xml = self.entry_xml(&sheet_path)?;
        let existing = xml_element_by_attribute(&sheet_xml, "hyperlink", "ref", &reference)?;
        let (updated_sheet_xml, _) =
            remove_xml_element_by_attribute(&sheet_xml, "hyperlink", "ref", &reference)?;
        sheet_xml = updated_sheet_xml;

        let mut relationships_xml = self.entry_xml(&relationships_path).unwrap_or_else(|_| {
            concat!(
                "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>",
                "<Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\"></Relationships>"
            )
            .to_owned()
        });
        if let Some(existing_id) = existing
            .as_deref()
            .and_then(|element| attribute_value(element, "r:id"))
        {
            let (updated, _) = remove_xml_element_by_attribute(
                &relationships_xml,
                "Relationship",
                "Id",
                existing_id,
            )?;
            relationships_xml = updated;
        }

        let hyperlink_element = match hyperlink.hyperlink_type {
            TemplateHyperlinkType::Document => {
                let location = hyperlink.hyperlink_type.package_target(&hyperlink.address);
                format!(
                    "<hyperlink ref=\"{}\" location=\"{}\"/>",
                    escape_xml(&reference),
                    escape_xml(&location)
                )
            }
            TemplateHyperlinkType::Url
            | TemplateHyperlinkType::Email
            | TemplateHyperlinkType::File => {
                let target = hyperlink.hyperlink_type.package_target(&hyperlink.address);
                let relationship_id = next_relationship_id(&relationships_xml);
                let relationship = format!(
                    concat!(
                        "<Relationship Id=\"{}\" ",
                        "Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/hyperlink\" ",
                        "Target=\"{}\" TargetMode=\"External\"/>"
                    ),
                    relationship_id,
                    escape_xml(&target)
                );
                relationships_xml =
                    insert_before_close_tag(&relationships_xml, "</Relationships>", &relationship)?;
                format!(
                    "<hyperlink ref=\"{}\" r:id=\"{}\"/>",
                    escape_xml(&reference),
                    relationship_id
                )
            }
        };

        sheet_xml = upsert_hyperlink_element(&sheet_xml, &hyperlink_element)?;
        self.entry_mut(&sheet_path)?.bytes = sheet_xml.into_bytes();
        if let Ok(entry) = self.entry_mut(&relationships_path) {
            entry.bytes = relationships_xml.into_bytes();
        } else if hyperlink.hyperlink_type != TemplateHyperlinkType::Document {
            self.entries.push(OoxmlZipEntry {
                name: relationships_path,
                is_dir: false,
                compression: CompressionMethod::Deflated,
                unix_mode: None,
                bytes: relationships_xml.into_bytes(),
            });
        }
        Ok(())
    }

    /// 按模板实际行高、列宽和 Java 锚点语义插入图片。
    ///
    /// 图片先由统一 generation 后端编译，再把 drawing、relationship 与 media 合并到
    /// 原模板包；未知部件保持原样。任一阶段失败都会恢复调用前快照。
    pub fn set_template_image(
        &mut self,
        sheet_name: &str,
        row_index: u32,
        column_index: u16,
        image: &TemplateImage,
    ) -> Result<()> {
        let snapshot = self.clone();
        if let Err(error) =
            self.set_template_image_inner(sheet_name, row_index, column_index, image)
        {
            *self = snapshot;
            return Err(error);
        }
        Ok(())
    }

    fn set_template_image_inner(
        &mut self,
        sheet_name: &str,
        row_index: u32,
        column_index: u16,
        image: &TemplateImage,
    ) -> Result<()> {
        let sheet_path = self.worksheet_path_by_name(sheet_name)?;
        let sheet_xml = self.entry_xml(&sheet_path)?;
        let resolved = super::resolve_image_anchor(
            super::ImageAnchorSpec {
                current_row: row_index,
                current_column: column_index,
                first_row: image.first_row,
                first_column: image.first_column,
                last_row: image.last_row,
                last_column: image.last_column,
                left: image.left,
                right: image.right,
                top: image.top,
                bottom: image.bottom,
            },
            |column| template_column_width_pixels(&sheet_xml, column),
            |row| template_row_height_pixels(&sheet_xml, row),
        )?;
        let movement = match image.movement {
            TemplateImageMovement::MoveAndResize => {
                super::generation::ObjectMovement::MoveAndSizeWithCells
            }
            TemplateImageMovement::MoveDontResize => {
                super::generation::ObjectMovement::MoveButDontSizeWithCells
            }
            TemplateImageMovement::DontMoveOrResize => {
                super::generation::ObjectMovement::DontMoveOrSizeWithCells
            }
        };
        let mut compiler = super::generation::new_workbook();
        let worksheet = super::generation::create_worksheet(&mut compiler, sheet_name, false)?;
        super::generation::insert_scaled_image(
            worksheet,
            resolved.first_row,
            resolved.first_column,
            &image.bytes,
            resolved.width,
            resolved.height,
            movement,
            resolved.left,
            resolved.top,
        )?;
        let compiled = super::generation::serialize_workbook(&mut compiler)?;
        self.import_image(&compiled, sheet_name)
    }

    fn import_image(&mut self, compiled_xlsx: &[u8], sheet_name: &str) -> Result<()> {
        let source = Self::from_bytes(compiled_xlsx)?;
        let source_sheet_path = source.worksheet_path_by_name(sheet_name)?;
        let source_sheet_xml = source.entry_xml(&source_sheet_path)?;
        let source_drawing_id = drawing_relationship_id(&source_sheet_xml)
            .ok_or_else(|| Error::Xlsx("compiled image worksheet has no drawing".to_owned()))?;
        let source_sheet_rels_path = relationship_part_name(&source_sheet_path);
        let source_sheet_rels = source.entry_xml(&source_sheet_rels_path)?;
        let source_drawing_target = relationship_target(&source_sheet_rels, &source_drawing_id)
            .ok_or_else(|| {
                Error::Xlsx("compiled image drawing relationship is missing".to_owned())
            })?;
        let source_drawing_path = resolve_target(&source_sheet_path, &source_drawing_target)?;
        let source_drawing_xml = source.entry_xml(&source_drawing_path)?;
        let source_anchor = image_anchor(&source_drawing_xml)
            .ok_or_else(|| Error::Xlsx("compiled drawing has no image anchor".to_owned()))?;
        let source_image_id = drawing_image_relationship_id(&source_anchor).ok_or_else(|| {
            Error::Xlsx("compiled image anchor has no relationship id".to_owned())
        })?;
        let source_drawing_rels_path = relationship_part_name(&source_drawing_path);
        let source_drawing_rels = source.entry_xml(&source_drawing_rels_path)?;
        let source_image_target = relationship_target(&source_drawing_rels, &source_image_id)
            .ok_or_else(|| Error::Xlsx("compiled image relationship is missing".to_owned()))?;
        let source_image_path = resolve_target(&source_drawing_path, &source_image_target)?;
        let source_image = source
            .entries
            .iter()
            .find(|entry| entry.name.eq_ignore_ascii_case(&source_image_path))
            .ok_or_else(|| Error::Xlsx("compiled image media part is missing".to_owned()))?;
        let extension = Path::new(&source_image_path)
            .extension()
            .and_then(|value| value.to_str())
            .map(str::to_ascii_lowercase)
            .ok_or_else(|| Error::Xlsx("compiled image media has no extension".to_owned()))?;
        let suffix = format!(".{extension}");
        let image_number = next_part_number(&self.entries, "xl/media/image", &suffix);
        let image_path = format!("xl/media/image{image_number}.{extension}");
        self.entries.push(OoxmlZipEntry {
            name: image_path,
            is_dir: false,
            compression: source_image.compression,
            unix_mode: source_image.unix_mode,
            bytes: source_image.bytes.clone(),
        });

        let target_sheet_path = self.worksheet_path_by_name(sheet_name)?;
        let target_sheet_rels_path = relationship_part_name(&target_sheet_path);
        let target_sheet_xml = self.entry_xml(&target_sheet_path)?;
        if let Some(target_drawing_id) = drawing_relationship_id(&target_sheet_xml) {
            let target_sheet_rels = self.entry_xml(&target_sheet_rels_path)?;
            let target_drawing_target = relationship_target(&target_sheet_rels, &target_drawing_id)
                .ok_or_else(|| {
                    Error::Xlsx("template drawing relationship is missing".to_owned())
                })?;
            let target_drawing_path = resolve_target(&target_sheet_path, &target_drawing_target)?;
            let target_drawing_rels_path = relationship_part_name(&target_drawing_path);
            let mut target_drawing_rels = self.entry_xml(&target_drawing_rels_path)?;
            let new_image_id = next_relationship_id(&target_drawing_rels);
            let drawing_xml = self.entry_xml(&target_drawing_path)?;
            let imported_anchor = with_next_drawing_object_id(
                &drawing_xml,
                &source_anchor.replace(
                    &format!("r:embed=\"{source_image_id}\""),
                    &format!("r:embed=\"{new_image_id}\""),
                ),
            )?;
            self.entry_mut(&target_drawing_path)?.bytes =
                insert_before_close_tag(&drawing_xml, "</xdr:wsDr>", &imported_anchor)?
                    .into_bytes();
            let relationship = format!(
                "<Relationship Id=\"{new_image_id}\" \
                 Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/image\" \
                 Target=\"../media/image{image_number}.{extension}\"/>"
            );
            target_drawing_rels =
                insert_before_close_tag(&target_drawing_rels, "</Relationships>", &relationship)?;
            self.entry_mut(&target_drawing_rels_path)?.bytes = target_drawing_rels.into_bytes();
        } else {
            let drawing_number = next_part_number(&self.entries, "xl/drawings/drawing", ".xml");
            let drawing_path = format!("xl/drawings/drawing{drawing_number}.xml");
            let drawing_rels_path = relationship_part_name(&drawing_path);
            let drawing_xml = source_drawing_xml.replace(
                &format!("r:embed=\"{source_image_id}\""),
                "r:embed=\"rId1\"",
            );
            let drawing_rels = format!(
                "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
                 <Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\">\
                 <Relationship Id=\"rId1\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/image\" \
                 Target=\"../media/image{image_number}.{extension}\"/>\
                 </Relationships>"
            );
            self.entries.push(OoxmlZipEntry {
                name: drawing_path.clone(),
                is_dir: false,
                compression: CompressionMethod::Deflated,
                unix_mode: None,
                bytes: drawing_xml.into_bytes(),
            });
            self.entries.push(OoxmlZipEntry {
                name: drawing_rels_path,
                is_dir: false,
                compression: CompressionMethod::Deflated,
                unix_mode: None,
                bytes: drawing_rels.into_bytes(),
            });
            let mut sheet_rels = self.entry_xml(&target_sheet_rels_path).unwrap_or_else(|_| {
                concat!(
                    "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>",
                    "<Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\"></Relationships>"
                )
                .to_owned()
            });
            let drawing_id = next_relationship_id(&sheet_rels);
            let relationship = format!(
                "<Relationship Id=\"{drawing_id}\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/drawing\" Target=\"../drawings/drawing{drawing_number}.xml\"/>"
            );
            sheet_rels = insert_before_close_tag(&sheet_rels, "</Relationships>", &relationship)?;
            if let Ok(entry) = self.entry_mut(&target_sheet_rels_path) {
                entry.bytes = sheet_rels.into_bytes();
            } else {
                self.entries.push(OoxmlZipEntry {
                    name: target_sheet_rels_path,
                    is_dir: false,
                    compression: CompressionMethod::Deflated,
                    unix_mode: None,
                    bytes: sheet_rels.into_bytes(),
                });
            }
            let sheet_xml = self.entry_xml(&target_sheet_path)?;
            self.entry_mut(&target_sheet_path)?.bytes =
                insert_drawing_reference(&sheet_xml, &drawing_id)?.into_bytes();
            ensure_content_type_override(
                &mut self.entries,
                &format!("/{drawing_path}"),
                "application/vnd.openxmlformats-officedocument.drawing+xml",
            )?;
        }
        ensure_content_type_default(
            &mut self.entries,
            &extension,
            image_content_type(&extension)?,
        )?;
        Ok(())
    }

    fn import_comment_inner(
        &mut self,
        compiled_xlsx: &[u8],
        sheet_name: &str,
        row_index: u32,
        column_index: u16,
    ) -> Result<()> {
        let source = Self::from_bytes(compiled_xlsx)?;
        let source_sheet_path = source.worksheet_path_by_name(sheet_name)?;
        let source_rels_path = relationship_part_name(&source_sheet_path);
        let source_rels = source.entry_xml(&source_rels_path)?;
        let source_comments_target = relationship_target_by_type(&source_rels, "/comments")
            .ok_or_else(|| Error::Xlsx("compiled comment relationship is missing".to_owned()))?;
        let source_vml_target = relationship_target_by_type(&source_rels, "/vmlDrawing")
            .ok_or_else(|| {
                Error::Xlsx("compiled comment VML relationship is missing".to_owned())
            })?;
        let source_comments_path = resolve_target(&source_sheet_path, &source_comments_target)?;
        let source_vml_path = resolve_target(&source_sheet_path, &source_vml_target)?;
        let source_comments = source.entry_xml(&source_comments_path)?;
        let source_vml = source.entry_xml(&source_vml_path)?;
        let reference = a1_reference(row_index, column_index);
        let source_comment =
            xml_element_by_attribute(&source_comments, "comment", "ref", &reference)?
                .ok_or_else(|| Error::Xlsx(format!("compiled comment {reference} is missing")))?;
        let source_shape = vml_comment_shape(
            &source_vml,
            usize::try_from(row_index).unwrap_or(usize::MAX),
            usize::from(column_index),
        )?
        .ok_or_else(|| Error::Xlsx(format!("compiled VML shape {reference} is missing")))?;

        let target_sheet_path = self.worksheet_path_by_name(sheet_name)?;
        let target_rels_path = relationship_part_name(&target_sheet_path);
        let mut target_rels = self.entry_xml(&target_rels_path).unwrap_or_else(|_| {
            concat!(
                "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>",
                "<Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\"></Relationships>"
            )
            .to_owned()
        });
        let existing_comments_target = relationship_target_by_type(&target_rels, "/comments");
        self.remove_comment(sheet_name, row_index, column_index)?;

        if let Some(comments_target) = existing_comments_target {
            let comments_path = resolve_target(&target_sheet_path, &comments_target)?;
            let vml_target =
                relationship_target_by_type(&target_rels, "/vmlDrawing").ok_or_else(|| {
                    Error::Xlsx("template comments have no VML relationship".to_owned())
                })?;
            let vml_path = resolve_target(&target_sheet_path, &vml_target)?;
            let comments_xml = self.entry_xml(&comments_path)?;
            let source_author_id = attribute_value(&source_comment, "authorId")
                .and_then(|value| value.parse::<usize>().ok())
                .ok_or_else(|| Error::Xlsx("compiled comment authorId is invalid".to_owned()))?;
            let source_author =
                xml_element_inner_by_index(&source_comments, "author", source_author_id)
                    .ok_or_else(|| Error::Xlsx("compiled comment author is missing".to_owned()))?;
            let target_authors = xml_element_inners(&comments_xml, "author");
            let (comments_xml, target_author_id) = if let Some(index) = target_authors
                .iter()
                .position(|author| *author == source_author)
            {
                (comments_xml, index)
            } else {
                (
                    insert_before_close_tag(
                        &comments_xml,
                        "</authors>",
                        &format!("<author>{source_author}</author>"),
                    )?,
                    target_authors.len(),
                )
            };
            let source_comment =
                replace_xml_attribute(&source_comment, "authorId", &target_author_id.to_string())?;
            self.entry_mut(&comments_path)?.bytes =
                insert_before_close_tag(&comments_xml, "</commentList>", &source_comment)?
                    .into_bytes();

            let vml_xml = self.entry_xml(&vml_path)?;
            let source_shape = with_next_vml_shape_id(&vml_xml, &source_shape)?;
            self.entry_mut(&vml_path)?.bytes =
                insert_before_close_tag(&vml_xml, "</xml>", &source_shape)?.into_bytes();
            return Ok(());
        }

        let comments_number = next_part_number(&self.entries, "xl/comments", ".xml");
        let vml_number = next_part_number(&self.entries, "xl/drawings/vmlDrawing", ".vml");
        let comments_path = format!("xl/comments{comments_number}.xml");
        let vml_path = format!("xl/drawings/vmlDrawing{vml_number}.vml");
        self.entries.push(OoxmlZipEntry {
            name: comments_path.clone(),
            is_dir: false,
            compression: CompressionMethod::Deflated,
            unix_mode: None,
            bytes: source_comments.into_bytes(),
        });
        self.entries.push(OoxmlZipEntry {
            name: vml_path,
            is_dir: false,
            compression: CompressionMethod::Deflated,
            unix_mode: None,
            bytes: source_vml.into_bytes(),
        });
        let comments_id = next_relationship_id(&target_rels);
        let comments_relationship = format!(
            "<Relationship Id=\"{comments_id}\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/comments\" Target=\"../comments{comments_number}.xml\"/>"
        );
        target_rels =
            insert_before_close_tag(&target_rels, "</Relationships>", &comments_relationship)?;
        let vml_id = next_relationship_id(&target_rels);
        let vml_relationship = format!(
            "<Relationship Id=\"{vml_id}\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/vmlDrawing\" Target=\"../drawings/vmlDrawing{vml_number}.vml\"/>"
        );
        target_rels = insert_before_close_tag(&target_rels, "</Relationships>", &vml_relationship)?;
        if let Ok(entry) = self.entry_mut(&target_rels_path) {
            entry.bytes = target_rels.into_bytes();
        } else {
            self.entries.push(OoxmlZipEntry {
                name: target_rels_path,
                is_dir: false,
                compression: CompressionMethod::Deflated,
                unix_mode: None,
                bytes: target_rels.into_bytes(),
            });
        }
        let sheet_xml = self.entry_xml(&target_sheet_path)?;
        self.entry_mut(&target_sheet_path)?.bytes = insert_before_close_tag(
            &sheet_xml,
            "</worksheet>",
            &format!("<legacyDrawing r:id=\"{vml_id}\"/>"),
        )?
        .into_bytes();
        ensure_content_type_override(
            &mut self.entries,
            &format!("/{comments_path}"),
            "application/vnd.openxmlformats-officedocument.spreadsheetml.comments+xml",
        )?;
        ensure_content_type_default(
            &mut self.entries,
            "vml",
            "application/vnd.openxmlformats-officedocument.vmlDrawing",
        )?;
        Ok(())
    }

    /// 为模板工作表添加兼容 Excel 的传统密码保护。
    ///
    /// # Errors
    ///
    /// 工作表不存在或工作表 XML 损坏时返回错误。
    pub fn protect_sheet(&mut self, sheet_name: &str, password: &str) -> Result<()> {
        let path = self.worksheet_path_by_name(sheet_name)?;
        let entry = self.entry_mut(&path)?;
        let xml = String::from_utf8(std::mem::take(&mut entry.bytes))
            .map_err(|error| Error::Xlsx(error.to_string()))?;
        entry.bytes = apply_sheet_protection(&xml, password)?.into_bytes();
        Ok(())
    }

    /// 将一个编译工作簿中的单个图表移植到模板工作表，同时保留原包未知部件。
    ///
    /// 已存在 drawing 时追加 anchor/relationship；不存在时创建新的 drawing 部件。
    ///
    /// # Errors
    ///
    /// 编译工作簿缺少图表关系、目标工作表不存在或 OOXML 关系损坏时返回错误。
    pub fn import_chart(&mut self, compiled_xlsx: &[u8], sheet_name: &str) -> Result<()> {
        let source = Self::from_bytes(compiled_xlsx)?;
        let source_sheet_path = source.worksheet_path_by_name(sheet_name)?;
        let source_sheet_xml = source.entry_xml(&source_sheet_path)?;
        let source_drawing_id = drawing_relationship_id(&source_sheet_xml)
            .ok_or_else(|| Error::Xlsx("compiled chart worksheet has no drawing".to_owned()))?;
        let source_sheet_rels_path = relationship_part_name(&source_sheet_path);
        let source_sheet_rels = source.entry_xml(&source_sheet_rels_path)?;
        let source_drawing_target = relationship_target(&source_sheet_rels, &source_drawing_id)
            .ok_or_else(|| {
                Error::Xlsx("compiled chart drawing relationship is missing".to_owned())
            })?;
        let source_drawing_path = resolve_target(&source_sheet_path, &source_drawing_target)?;
        let source_drawing_xml = source.entry_xml(&source_drawing_path)?;
        let source_anchor = chart_anchor(&source_drawing_xml)
            .ok_or_else(|| Error::Xlsx("compiled drawing has no chart anchor".to_owned()))?;
        let source_anchor_chart_id =
            drawing_chart_relationship_id(&source_anchor).ok_or_else(|| {
                Error::Xlsx("compiled chart anchor has no relationship id".to_owned())
            })?;
        let source_drawing_rels_path = relationship_part_name(&source_drawing_path);
        let source_drawing_rels = source.entry_xml(&source_drawing_rels_path)?;
        let source_chart_target =
            relationship_target(&source_drawing_rels, &source_anchor_chart_id).ok_or_else(
                || Error::Xlsx("compiled drawing chart relationship is missing".to_owned()),
            )?;
        let source_chart_path = resolve_target(&source_drawing_path, &source_chart_target)?;
        let source_chart = source
            .entries
            .iter()
            .find(|entry| entry.name.eq_ignore_ascii_case(&source_chart_path))
            .ok_or_else(|| Error::Xlsx("compiled chart part is missing".to_owned()))?
            .bytes
            .clone();

        let chart_number = next_part_number(&self.entries, "xl/charts/chart", ".xml");
        let chart_path = format!("xl/charts/chart{chart_number}.xml");
        let target_sheet_path = self.worksheet_path_by_name(sheet_name)?;
        let target_sheet_rels_path = relationship_part_name(&target_sheet_path);
        let target_sheet_xml = self.entry_xml(&target_sheet_path)?;

        self.entries.push(OoxmlZipEntry {
            name: chart_path.clone(),
            is_dir: false,
            compression: CompressionMethod::Deflated,
            unix_mode: None,
            bytes: source_chart,
        });

        if let Some(target_drawing_id) = drawing_relationship_id(&target_sheet_xml) {
            let target_sheet_rels = self.entry_xml(&target_sheet_rels_path)?;
            let target_drawing_target = relationship_target(&target_sheet_rels, &target_drawing_id)
                .ok_or_else(|| {
                    Error::Xlsx("template drawing relationship is missing".to_owned())
                })?;
            let target_drawing_path = resolve_target(&target_sheet_path, &target_drawing_target)?;
            let target_drawing_rels_path = relationship_part_name(&target_drawing_path);
            let mut target_drawing_rels = self.entry_xml(&target_drawing_rels_path)?;
            let new_chart_id = next_relationship_id(&target_drawing_rels);
            let imported_anchor = source_anchor.replace(
                &format!("r:id=\"{source_anchor_chart_id}\""),
                &format!("r:id=\"{new_chart_id}\""),
            );
            let drawing_entry = self.entry_mut(&target_drawing_path)?;
            let drawing_xml = String::from_utf8(std::mem::take(&mut drawing_entry.bytes))
                .map_err(|error| Error::Xlsx(error.to_string()))?;
            drawing_entry.bytes =
                insert_before_close_tag(&drawing_xml, "</xdr:wsDr>", &imported_anchor)?
                    .into_bytes();
            let relationship = format!(
                "<Relationship Id=\"{new_chart_id}\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/chart\" Target=\"../charts/chart{chart_number}.xml\"/>"
            );
            target_drawing_rels =
                insert_before_close_tag(&target_drawing_rels, "</Relationships>", &relationship)?;
            self.entry_mut(&target_drawing_rels_path)?.bytes = target_drawing_rels.into_bytes();
        } else {
            let drawing_number = next_part_number(&self.entries, "xl/drawings/drawing", ".xml");
            let drawing_path = format!("xl/drawings/drawing{drawing_number}.xml");
            let drawing_rels_path = relationship_part_name(&drawing_path);
            let drawing_xml = source_drawing_xml.replace(
                &format!("r:id=\"{source_anchor_chart_id}\""),
                "r:id=\"rId1\"",
            );
            let drawing_rels = format!(
                "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
                 <Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\">\
                 <Relationship Id=\"rId1\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/chart\" \
                 Target=\"../charts/chart{chart_number}.xml\"/>\
                 </Relationships>"
            );
            self.entries.push(OoxmlZipEntry {
                name: drawing_path.clone(),
                is_dir: false,
                compression: CompressionMethod::Deflated,
                unix_mode: None,
                bytes: drawing_xml.into_bytes(),
            });
            self.entries.push(OoxmlZipEntry {
                name: drawing_rels_path,
                is_dir: false,
                compression: CompressionMethod::Deflated,
                unix_mode: None,
                bytes: drawing_rels.into_bytes(),
            });

            let mut sheet_rels = self.entry_xml(&target_sheet_rels_path).unwrap_or_else(|_| {
                concat!(
                    "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>",
                    "<Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\"></Relationships>"
                )
                .to_owned()
            });
            let drawing_id = next_relationship_id(&sheet_rels);
            let relationship = format!(
                "<Relationship Id=\"{drawing_id}\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/drawing\" Target=\"../drawings/drawing{drawing_number}.xml\"/>"
            );
            sheet_rels = insert_before_close_tag(&sheet_rels, "</Relationships>", &relationship)?;
            if let Ok(entry) = self.entry_mut(&target_sheet_rels_path) {
                entry.bytes = sheet_rels.into_bytes();
            } else {
                self.entries.push(OoxmlZipEntry {
                    name: target_sheet_rels_path,
                    is_dir: false,
                    compression: CompressionMethod::Deflated,
                    unix_mode: None,
                    bytes: sheet_rels.into_bytes(),
                });
            }
            let sheet_entry = self.entry_mut(&target_sheet_path)?;
            let sheet_xml = String::from_utf8(std::mem::take(&mut sheet_entry.bytes))
                .map_err(|error| Error::Xlsx(error.to_string()))?;
            sheet_entry.bytes = insert_drawing_reference(&sheet_xml, &drawing_id)?.into_bytes();
            ensure_content_type_override(
                &mut self.entries,
                &format!("/{drawing_path}"),
                "application/vnd.openxmlformats-officedocument.drawing+xml",
            )?;
        }
        ensure_content_type_override(
            &mut self.entries,
            &format!("/{chart_path}"),
            "application/vnd.openxmlformats-officedocument.drawingml.chart+xml",
        )?;
        Ok(())
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 合并编译工作簿的样式表并返回目标 XF 索引。
    ///
    /// # Errors
    ///
    /// 底层 OOXML、ZIP、XML 或目标 I/O 操作失败，或输入不符合格式约束时返回错误。
    pub fn import_compiled_styles(
        &mut self,
        compiled_xlsx: &[u8],
        style_count: usize,
    ) -> Result<Vec<u32>> {
        if style_count == 0 {
            return Ok(Vec::new());
        }
        let compiled = Self::from_bytes(compiled_xlsx)?;
        let source_styles = compiled.entry_xml(STYLES_PATH)?;
        let (_, source_sheet_path) = compiled.worksheet_path_by_index(0)?;
        let source_sheet = compiled.entry_xml(&source_sheet_path)?;
        let source_indexes = (1..=style_count)
            .map(|row| {
                cell_style_index(&source_sheet, &format!("A{row}")).ok_or_else(|| {
                    Error::Xlsx(format!("compiled style cell A{row} has no style index"))
                })
            })
            .collect::<Result<Vec<_>>>()?;
        let destination = self.entry_mut(STYLES_PATH)?;
        let destination_styles = String::from_utf8(std::mem::take(&mut destination.bytes))
            .map_err(|error| Error::Xlsx(error.to_string()))?;
        let (updated, mapped) =
            merge_compiled_styles(&destination_styles, &source_styles, &source_indexes)?;
        destination.bytes = updated.into_bytes();
        Ok(mapped)
    }

    /// 将编译样式叠加到指定模板 XF，并返回新样式索引。
    ///
    /// # Errors
    ///
    /// 当编译工作簿、模板样式表或样式索引无效时返回错误。
    pub fn import_compiled_styles_onto(
        &mut self,
        compiled_xlsx: &[u8],
        base_indexes: &[usize],
    ) -> Result<Vec<u32>> {
        if base_indexes.is_empty() {
            return Ok(Vec::new());
        }
        let compiled = Self::from_bytes(compiled_xlsx)?;
        let source_styles = compiled.entry_xml(STYLES_PATH)?;
        let (_, source_sheet_path) = compiled.worksheet_path_by_index(0)?;
        let source_sheet = compiled.entry_xml(&source_sheet_path)?;
        let source_indexes = (1..=base_indexes.len())
            .map(|row| {
                cell_style_index(&source_sheet, &format!("A{row}")).ok_or_else(|| {
                    Error::Xlsx(format!("compiled style cell A{row} has no style index"))
                })
            })
            .collect::<Result<Vec<_>>>()?;
        let destination = self.entry_mut(STYLES_PATH)?;
        let destination_styles = String::from_utf8(std::mem::take(&mut destination.bytes))
            .map_err(|error| Error::Xlsx(error.to_string()))?;
        let (updated, mapped) = merge_compiled_styles_onto(
            &destination_styles,
            &source_styles,
            &source_indexes,
            base_indexes,
        )?;
        destination.bytes = updated.into_bytes();
        Ok(mapped)
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 序列化为 XLSX 字节。
    ///
    /// # Errors
    ///
    /// 底层 OOXML、ZIP、XML 或目标 I/O 操作失败，或输入不符合格式约束时返回错误。
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        self.entries.to_bytes()
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 保存到路径。
    ///
    /// # Errors
    ///
    /// 底层 OOXML、ZIP、XML 或目标 I/O 操作失败，或输入不符合格式约束时返回错误。
    pub fn save_to_path(&self, path: &Path) -> Result<()> {
        self.entries.save_to_path(path)
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 保存到输出流。
    ///
    /// # Errors
    ///
    /// 底层 OOXML、ZIP、XML 或目标 I/O 操作失败，或输入不符合格式约束时返回错误。
    pub fn save_to_writer(&self, output: &mut dyn Write) -> Result<()> {
        self.entries.save_to_writer(output)
    }

    fn workbook_sheets(&self) -> Result<Vec<(String, String)>> {
        let xml = self.entry_xml(WORKBOOK_PATH)?;
        Ok(xml_elements(&xml, "sheet")
            .filter_map(|element| {
                Some((
                    attribute_value(element, "name")?.to_owned(),
                    attribute_value(element, "r:id")?.to_owned(),
                ))
            })
            .collect())
    }

    fn worksheet_part_for_relationship(
        &self,
        relationship_id: &str,
        sheet_name: &str,
    ) -> Result<String> {
        let xml = self.entry_xml(WORKBOOK_RELS_PATH)?;
        let target = xml_elements(&xml, "Relationship")
            .find(|element| attribute_value(element, "Id") == Some(relationship_id))
            .and_then(|element| attribute_value(element, "Target"))
            .ok_or_else(|| {
                Error::Xlsx(format!(
                    "workbook relationship {relationship_id} for sheet {sheet_name} is missing"
                ))
            })?;
        let normalized = resolve_target(WORKBOOK_PATH, target)?;
        self.entries
            .iter()
            .find(|entry| entry.name.eq_ignore_ascii_case(&normalized))
            .map(|entry| entry.name.clone())
            .ok_or_else(|| {
                Error::Xlsx(format!(
                    "worksheet part {normalized} for sheet {sheet_name} is missing"
                ))
            })
    }

    fn entry_xml(&self, path: &str) -> Result<String> {
        let entry = self
            .entries
            .iter()
            .find(|entry| entry.name.eq_ignore_ascii_case(path))
            .ok_or_else(|| Error::Xlsx(format!("template does not contain {path}")))?;
        entry_string(entry)
    }

    fn entry_mut(&mut self, path: &str) -> Result<&mut OoxmlZipEntry> {
        self.entries
            .iter_mut()
            .find(|entry| entry.name.eq_ignore_ascii_case(path))
            .ok_or_else(|| Error::Xlsx(format!("template does not contain {path}")))
    }
}

impl Deref for OoxmlTemplatePackage {
    type Target = OoxmlPackage;

    fn deref(&self) -> &Self::Target {
        &self.entries
    }
}

impl DerefMut for OoxmlTemplatePackage {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.entries
    }
}

fn validate_row_shapes(
    rows: &[Vec<(usize, TemplateCellValue)>],
    row_heights: &[Option<u16>],
    cell_styles: &[Vec<Option<u32>>],
    absent_rows: &[bool],
) -> Result<()> {
    if !absent_rows.is_empty() && absent_rows.len() != rows.len() {
        return Err(Error::Xlsx(
            "template absent-row count does not match appended row count".to_owned(),
        ));
    }
    if !row_heights.is_empty() && row_heights.len() != rows.len() {
        return Err(Error::Xlsx(
            "template row-height count does not match appended row count".to_owned(),
        ));
    }
    if !cell_styles.is_empty()
        && (cell_styles.len() != rows.len()
            || cell_styles
                .iter()
                .zip(rows)
                .any(|(styles, row)| styles.len() != row.len()))
    {
        return Err(Error::Xlsx(
            "template cell-style shape does not match appended rows".to_owned(),
        ));
    }
    Ok(())
}

fn entry_index(entries: &[OoxmlZipEntry], path: &str) -> Result<usize> {
    entries
        .iter()
        .position(|entry| entry.name.eq_ignore_ascii_case(path))
        .ok_or_else(|| Error::Xlsx(format!("template missing {path}")))
}

fn entry_string(entry: &OoxmlZipEntry) -> Result<String> {
    String::from_utf8(entry.bytes.clone()).map_err(|error| Error::Xlsx(error.to_string()))
}

fn xml_elements<'a>(xml: &'a str, tag: &'a str) -> impl Iterator<Item = &'a str> + 'a {
    let open = format!("<{tag}");
    let mut offset = 0;
    std::iter::from_fn(move || {
        let relative = xml[offset..].find(&open)?;
        let start = offset + relative;
        let end = start + xml[start..].find('>')? + 1;
        offset = end;
        Some(&xml[start..end])
    })
}

fn next_worksheet_part_name(entries: &[OoxmlZipEntry]) -> String {
    let maximum = entries
        .iter()
        .filter_map(|entry| {
            let lower = entry.name.to_ascii_lowercase();
            let rest = lower.strip_prefix("xl/worksheets/sheet")?;
            rest.chars()
                .take_while(char::is_ascii_digit)
                .collect::<String>()
                .parse::<usize>()
                .ok()
        })
        .max()
        .unwrap_or(0);
    format!("xl/worksheets/sheet{}.xml", maximum.saturating_add(1))
}

fn next_relationship_id(xml: &str) -> String {
    format!("rId{}", next_numeric_attribute(xml, "Id=\"rId"))
}

fn next_sheet_id(xml: &str) -> usize {
    xml_elements(xml, "sheet")
        .filter_map(|element| attribute_value(element, "sheetId")?.parse::<usize>().ok())
        .max()
        .unwrap_or(0)
        .saturating_add(1)
}

fn next_numeric_attribute(xml: &str, marker: &str) -> usize {
    let mut maximum = 0;
    let mut offset = 0;
    while let Some(relative) = xml[offset..].find(marker) {
        let start = offset + relative + marker.len();
        let digits = xml[start..]
            .chars()
            .take_while(char::is_ascii_digit)
            .collect::<String>();
        if let Ok(value) = digits.parse::<usize>() {
            maximum = maximum.max(value);
        }
        offset = start;
    }
    maximum.saturating_add(1)
}

fn insert_before_close_tag(xml: &str, close_tag: &str, fragment: &str) -> Result<String> {
    let index = xml
        .find(close_tag)
        .ok_or_else(|| Error::Xlsx(format!("template XML is missing {close_tag}")))?;
    Ok(format!("{}{}{}", &xml[..index], fragment, &xml[index..]))
}

fn upsert_hyperlink_element(xml: &str, hyperlink: &str) -> Result<String> {
    if xml.contains("</hyperlinks>") {
        return insert_before_close_tag(xml, "</hyperlinks>", hyperlink);
    }
    if let Some(start) = xml.find("<hyperlinks") {
        let end = start
            + xml[start..].find("/>").ok_or_else(|| {
                Error::Xlsx("template hyperlinks element is malformed".to_owned())
            })?
            + 2;
        return Ok(format!(
            "{}<hyperlinks>{hyperlink}</hyperlinks>{}",
            &xml[..start],
            &xml[end..]
        ));
    }
    // ECMA-376 CT_Worksheet 中 hyperlinks 位于 mergeCells 之后、打印和 drawing
    // 元素之前；选择所有后继元素中的最早位置以保持 schema 顺序。
    let insertion = [
        "<printOptions",
        "<pageMargins",
        "<pageSetup",
        "<headerFooter",
        "<rowBreaks",
        "<colBreaks",
        "<customProperties",
        "<cellWatches",
        "<ignoredErrors",
        "<smartTags",
        "<drawing",
        "<legacyDrawing",
        "<legacyDrawingHF",
        "<picture",
        "<oleObjects",
        "<controls",
        "<webPublishItems",
        "<tableParts",
        "<extLst",
        "</worksheet>",
    ]
    .into_iter()
    .filter_map(|marker| xml.find(marker))
    .min()
    .ok_or_else(|| Error::Xlsx("template XML is missing </worksheet>".to_owned()))?;
    Ok(format!(
        "{}<hyperlinks>{hyperlink}</hyperlinks>{}",
        &xml[..insertion],
        &xml[insertion..]
    ))
}

fn insert_drawing_reference(xml: &str, relationship_id: &str) -> Result<String> {
    let insertion = [
        "<legacyDrawing",
        "<legacyDrawingHF",
        "<picture",
        "<oleObjects",
        "<controls",
        "<webPublishItems",
        "<tableParts",
        "<extLst",
        "</worksheet>",
    ]
    .into_iter()
    .filter_map(|marker| xml.find(marker))
    .min()
    .ok_or_else(|| Error::Xlsx("template XML is missing </worksheet>".to_owned()))?;
    Ok(format!(
        "{}<drawing r:id=\"{}\"/>{}",
        &xml[..insertion],
        escape_xml(relationship_id),
        &xml[insertion..]
    ))
}

fn drawing_relationship_id(sheet_xml: &str) -> Option<String> {
    xml_elements(sheet_xml, "drawing")
        .next()
        .and_then(|element| attribute_value(element, "r:id"))
        .map(ToOwned::to_owned)
}

fn drawing_chart_relationship_id(anchor_xml: &str) -> Option<String> {
    xml_elements(anchor_xml, "c:chart")
        .next()
        .or_else(|| xml_elements(anchor_xml, "chart").next())
        .and_then(|element| attribute_value(element, "r:id"))
        .map(ToOwned::to_owned)
}

fn relationship_target(relationships_xml: &str, relationship_id: &str) -> Option<String> {
    xml_elements(relationships_xml, "Relationship")
        .find(|element| attribute_value(element, "Id") == Some(relationship_id))
        .and_then(|element| attribute_value(element, "Target"))
        .map(ToOwned::to_owned)
}

fn relationship_target_by_type(relationships_xml: &str, type_suffix: &str) -> Option<String> {
    xml_elements(relationships_xml, "Relationship")
        .find(|element| {
            attribute_value(element, "Type").is_some_and(|value| value.ends_with(type_suffix))
        })
        .and_then(|element| attribute_value(element, "Target"))
        .map(ToOwned::to_owned)
}

fn a1_reference(row_index: u32, column_index: u16) -> String {
    let mut column = usize::from(column_index).saturating_add(1);
    let mut letters = Vec::new();
    while column > 0 {
        column -= 1;
        letters.push(char::from(b'A' + u8::try_from(column % 26).unwrap_or(0)));
        column /= 26;
    }
    letters.reverse();
    format!(
        "{}{}",
        letters.into_iter().collect::<String>(),
        row_index.saturating_add(1)
    )
}

fn remove_xml_element_by_attribute(
    xml: &str,
    tag: &str,
    attribute: &str,
    expected: &str,
) -> Result<(String, bool)> {
    let marker = format!("<{tag}");
    let close = format!("</{tag}>");
    let mut offset = 0usize;
    while let Some(relative) = xml[offset..].find(&marker) {
        let start = offset + relative;
        let name_end = start + marker.len();
        if xml
            .as_bytes()
            .get(name_end)
            .is_some_and(|byte| !byte.is_ascii_whitespace())
        {
            offset = name_end;
            continue;
        }
        let open_end = start
            + xml[start..]
                .find('>')
                .ok_or_else(|| Error::Xlsx(format!("unterminated <{tag}> element")))?
            + 1;
        if attribute_value(&xml[start..open_end], attribute) != Some(expected) {
            offset = open_end;
            continue;
        }
        let end = if xml[start..open_end].trim_end().ends_with("/>") {
            open_end
        } else {
            open_end
                + xml[open_end..]
                    .find(&close)
                    .ok_or_else(|| Error::Xlsx(format!("missing {close}")))?
                + close.len()
        };
        let mut output = String::with_capacity(xml.len().saturating_sub(end - start));
        output.push_str(&xml[..start]);
        output.push_str(&xml[end..]);
        return Ok((output, true));
    }
    Ok((xml.to_owned(), false))
}

fn xml_element_by_attribute(
    xml: &str,
    tag: &str,
    attribute: &str,
    expected: &str,
) -> Result<Option<String>> {
    let marker = format!("<{tag}");
    let close = format!("</{tag}>");
    let mut offset = 0usize;
    while let Some(relative) = xml[offset..].find(&marker) {
        let start = offset + relative;
        let name_end = start + marker.len();
        if xml
            .as_bytes()
            .get(name_end)
            .is_some_and(|byte| !byte.is_ascii_whitespace())
        {
            offset = name_end;
            continue;
        }
        let open_end = start
            + xml[start..]
                .find('>')
                .ok_or_else(|| Error::Xlsx(format!("unterminated <{tag}> element")))?
            + 1;
        if attribute_value(&xml[start..open_end], attribute) != Some(expected) {
            offset = open_end;
            continue;
        }
        let end = if xml[start..open_end].trim_end().ends_with("/>") {
            open_end
        } else {
            open_end
                + xml[open_end..]
                    .find(&close)
                    .ok_or_else(|| Error::Xlsx(format!("missing {close}")))?
                + close.len()
        };
        return Ok(Some(xml[start..end].to_owned()));
    }
    Ok(None)
}

fn xml_element_inners<'a>(xml: &'a str, tag: &str) -> Vec<&'a str> {
    let marker = format!("<{tag}");
    let close = format!("</{tag}>");
    let mut values = Vec::new();
    let mut offset = 0usize;
    while let Some(relative) = xml[offset..].find(&marker) {
        let start = offset + relative;
        let name_end = start + marker.len();
        if xml
            .as_bytes()
            .get(name_end)
            .is_some_and(|byte| !byte.is_ascii_whitespace() && *byte != b'>')
        {
            offset = name_end;
            continue;
        }
        let Some(open_relative) = xml[start..].find('>') else {
            break;
        };
        let open_end = start + open_relative + 1;
        let Some(close_relative) = xml[open_end..].find(&close) else {
            break;
        };
        let end = open_end + close_relative;
        values.push(&xml[open_end..end]);
        offset = end + close.len();
    }
    values
}

fn xml_element_inner_by_index<'a>(xml: &'a str, tag: &str, index: usize) -> Option<&'a str> {
    xml_element_inners(xml, tag).get(index).copied()
}

fn replace_xml_attribute(xml: &str, attribute: &str, value: &str) -> Result<String> {
    let open_end = xml
        .find('>')
        .ok_or_else(|| Error::Xlsx("XML element has no closing bracket".to_owned()))?;
    let opening = &xml[..open_end];
    let marker = format!("{attribute}=\"");
    let start = opening
        .find(&marker)
        .ok_or_else(|| Error::Xlsx(format!("XML element has no {attribute} attribute")))?
        + marker.len();
    let end = start
        + opening[start..]
            .find('"')
            .ok_or_else(|| Error::Xlsx(format!("XML {attribute} attribute is unterminated")))?;
    Ok(format!("{}{}{}", &xml[..start], value, &xml[end..]))
}

fn vml_comment_shape(xml: &str, row: usize, column: usize) -> Result<Option<String>> {
    let mut offset = 0usize;
    while let Some(relative) = xml[offset..].find("<v:shape") {
        let start = offset + relative;
        let end = start
            + xml[start..]
                .find("</v:shape>")
                .ok_or_else(|| Error::Xlsx("VML shape has no closing tag".to_owned()))?
            + "</v:shape>".len();
        let shape = &xml[start..end];
        let matches_row = shape.contains(&format!("<x:Row>{row}</x:Row>"))
            || shape.contains(&format!("<Row>{row}</Row>"));
        let matches_column = shape.contains(&format!("<x:Column>{column}</x:Column>"))
            || shape.contains(&format!("<Column>{column}</Column>"));
        if matches_row && matches_column {
            return Ok(Some(shape.to_owned()));
        }
        offset = end;
    }
    Ok(None)
}

fn with_next_vml_shape_id(target_vml: &str, source_shape: &str) -> Result<String> {
    let open_end = source_shape
        .find('>')
        .ok_or_else(|| Error::Xlsx("compiled VML shape has no closing bracket".to_owned()))?;
    let source_id = attribute_value(&source_shape[..=open_end], "id")
        .ok_or_else(|| Error::Xlsx("compiled VML shape has no id".to_owned()))?;
    let target_id = format!(
        "_x0000_s{}",
        next_numeric_attribute(target_vml, "id=\"_x0000_s")
    );
    Ok(source_shape.replacen(
        &format!("id=\"{source_id}\""),
        &format!("id=\"{target_id}\""),
        1,
    ))
}

fn remove_vml_comment_shape(xml: &str, row: usize, column: usize) -> Result<(String, bool)> {
    let mut offset = 0usize;
    while let Some(relative) = xml[offset..].find("<v:shape") {
        let start = offset + relative;
        let end = start
            + xml[start..]
                .find("</v:shape>")
                .ok_or_else(|| Error::Xlsx("VML shape has no closing tag".to_owned()))?
            + "</v:shape>".len();
        let shape = &xml[start..end];
        let matches_row = shape.contains(&format!("<x:Row>{row}</x:Row>"))
            || shape.contains(&format!("<Row>{row}</Row>"));
        let matches_column = shape.contains(&format!("<x:Column>{column}</x:Column>"))
            || shape.contains(&format!("<Column>{column}</Column>"));
        if matches_row && matches_column {
            let mut output = String::with_capacity(xml.len().saturating_sub(end - start));
            output.push_str(&xml[..start]);
            output.push_str(&xml[end..]);
            return Ok((output, true));
        }
        offset = end;
    }
    Ok((xml.to_owned(), false))
}

fn chart_anchor(drawing_xml: &str) -> Option<String> {
    for tag in [
        "xdr:twoCellAnchor",
        "xdr:oneCellAnchor",
        "xdr:absoluteAnchor",
    ] {
        let open = format!("<{tag}");
        let Some(start) = drawing_xml.find(&open) else {
            continue;
        };
        let close = format!("</{tag}>");
        let end = drawing_xml[start..].find(&close)? + start + close.len();
        return Some(drawing_xml[start..end].to_owned());
    }
    None
}

fn image_anchor(drawing_xml: &str) -> Option<String> {
    for tag in [
        "xdr:twoCellAnchor",
        "xdr:oneCellAnchor",
        "xdr:absoluteAnchor",
    ] {
        let open = format!("<{tag}");
        let close = format!("</{tag}>");
        let mut offset = 0usize;
        while let Some(relative) = drawing_xml[offset..].find(&open) {
            let start = offset + relative;
            let end = drawing_xml[start..].find(&close)? + start + close.len();
            let anchor = &drawing_xml[start..end];
            if anchor.contains("<xdr:pic") || anchor.contains("<pic") {
                return Some(anchor.to_owned());
            }
            offset = end;
        }
    }
    None
}

fn drawing_image_relationship_id(anchor_xml: &str) -> Option<String> {
    xml_elements(anchor_xml, "a:blip")
        .next()
        .or_else(|| xml_elements(anchor_xml, "blip").next())
        .and_then(|element| attribute_value(element, "r:embed"))
        .map(ToOwned::to_owned)
}

fn with_next_drawing_object_id(target_drawing: &str, source_anchor: &str) -> Result<String> {
    let next_id = xml_elements(target_drawing, "xdr:cNvPr")
        .chain(xml_elements(target_drawing, "cNvPr"))
        .filter_map(|element| attribute_value(element, "id")?.parse::<usize>().ok())
        .max()
        .unwrap_or(0)
        .saturating_add(1);
    let source_property = xml_elements(source_anchor, "xdr:cNvPr")
        .next()
        .or_else(|| xml_elements(source_anchor, "cNvPr").next())
        .ok_or_else(|| Error::Xlsx("compiled image has no cNvPr metadata".to_owned()))?;
    let property = replace_xml_attribute(source_property, "id", &next_id.to_string())?;
    let property = replace_xml_attribute(&property, "name", &format!("Picture {next_id}"))?;
    Ok(source_anchor.replacen(source_property, &property, 1))
}

fn template_column_width_pixels(sheet_xml: &str, column: u16) -> u32 {
    let one_based = u32::from(column) + 1;
    let width = xml_elements(sheet_xml, "col")
        .find_map(|element| {
            let minimum = attribute_value(element, "min")?.parse::<u32>().ok()?;
            let maximum = attribute_value(element, "max")?.parse::<u32>().ok()?;
            (minimum <= one_based && one_based <= maximum)
                .then(|| attribute_value(element, "width")?.parse::<f64>().ok())
                .flatten()
        })
        .or_else(|| {
            xml_elements(sheet_xml, "sheetFormatPr")
                .next()
                .and_then(|element| attribute_value(element, "defaultColWidth"))
                .and_then(|value| value.parse::<f64>().ok())
        });
    width
        .filter(|value| value.is_finite() && *value > 0.0)
        .map_or(64, |value| (value.mul_add(7.0, 5.0)).round() as u32)
}

fn template_row_height_pixels(sheet_xml: &str, row: u32) -> u32 {
    let one_based = row + 1;
    let height = xml_elements(sheet_xml, "row")
        .find(|element| {
            attribute_value(element, "r").and_then(|value| value.parse::<u32>().ok())
                == Some(one_based)
        })
        .and_then(|element| attribute_value(element, "ht"))
        .and_then(|value| value.parse::<f64>().ok())
        .or_else(|| {
            xml_elements(sheet_xml, "sheetFormatPr")
                .next()
                .and_then(|element| attribute_value(element, "defaultRowHeight"))
                .and_then(|value| value.parse::<f64>().ok())
        });
    height
        .filter(|value| value.is_finite() && *value > 0.0)
        .map_or(20, |value| (value * 4.0 / 3.0).round() as u32)
}

fn image_content_type(extension: &str) -> Result<&'static str> {
    match extension {
        "png" => Ok("image/png"),
        "jpg" | "jpeg" => Ok("image/jpeg"),
        "gif" => Ok("image/gif"),
        "bmp" => Ok("image/bmp"),
        other => Err(Error::Xlsx(format!(
            "unsupported compiled image media extension: {other}"
        ))),
    }
}

fn next_part_number(entries: &[OoxmlZipEntry], prefix: &str, suffix: &str) -> usize {
    entries
        .iter()
        .filter_map(|entry| {
            entry
                .name
                .strip_prefix(prefix)?
                .strip_suffix(suffix)?
                .parse::<usize>()
                .ok()
        })
        .max()
        .unwrap_or(0)
        .saturating_add(1)
}

fn ensure_content_type_override(
    entries: &mut [OoxmlZipEntry],
    part_name: &str,
    content_type: &str,
) -> Result<()> {
    let entry = entries
        .iter_mut()
        .find(|entry| entry.name.eq_ignore_ascii_case(CONTENT_TYPES_PATH))
        .ok_or_else(|| Error::Xlsx("template missing [Content_Types].xml".to_owned()))?;
    let xml = String::from_utf8(std::mem::take(&mut entry.bytes))
        .map_err(|error| Error::Xlsx(error.to_string()))?;
    if xml.contains(&format!("PartName=\"{part_name}\"")) {
        entry.bytes = xml.into_bytes();
        return Ok(());
    }
    let override_tag =
        format!("<Override PartName=\"{part_name}\" ContentType=\"{content_type}\"/>");
    entry.bytes = insert_before_close_tag(&xml, "</Types>", &override_tag)?.into_bytes();
    Ok(())
}

fn ensure_content_type_default(
    entries: &mut [OoxmlZipEntry],
    extension: &str,
    content_type: &str,
) -> Result<()> {
    let entry = entries
        .iter_mut()
        .find(|entry| entry.name.eq_ignore_ascii_case(CONTENT_TYPES_PATH))
        .ok_or_else(|| Error::Xlsx("template missing [Content_Types].xml".to_owned()))?;
    let xml = String::from_utf8(std::mem::take(&mut entry.bytes))
        .map_err(|error| Error::Xlsx(error.to_string()))?;
    if xml.contains(&format!("Extension=\"{extension}\"")) {
        entry.bytes = xml.into_bytes();
        return Ok(());
    }
    let default_tag =
        format!("<Default Extension=\"{extension}\" ContentType=\"{content_type}\"/>");
    entry.bytes = insert_before_close_tag(&xml, "</Types>", &default_tag)?.into_bytes();
    Ok(())
}

fn blank_worksheet_with_inherited_format(entries: &[OoxmlZipEntry]) -> Vec<u8> {
    let Some(source) = entries.iter().find(|entry| {
        let lower = entry.name.to_ascii_lowercase();
        lower.starts_with("xl/worksheets/sheet") && lower.ends_with(".xml")
    }) else {
        return EMPTY_WORKSHEET_XML.as_bytes().to_vec();
    };
    let Ok(xml) = std::str::from_utf8(&source.bytes) else {
        return EMPTY_WORKSHEET_XML.as_bytes().to_vec();
    };
    let format = extract_xml_element(xml, "sheetFormatPr").unwrap_or_default();
    let columns = extract_xml_element(xml, "cols").unwrap_or_default();
    if format.is_empty() && columns.is_empty() {
        return EMPTY_WORKSHEET_XML.as_bytes().to_vec();
    }
    format!(
        concat!(
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>"#,
            r#"<worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main" "#,
            r#"xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">"#,
            r#"<dimension ref="A1"/>{format}{columns}<sheetData></sheetData></worksheet>"#
        ),
        format = format,
        columns = columns
    )
    .into_bytes()
}

fn extract_xml_element(xml: &str, tag: &str) -> Option<String> {
    let start = xml.find(&format!("<{tag}"))?;
    let rest = &xml[start..];
    let close = format!("</{tag}>");
    if let Some(close_at) = rest.find(&close) {
        return Some(rest[..close_at + close.len()].to_owned());
    }
    let self_close = rest.find("/>")?;
    if rest[..self_close].contains('>') {
        return None;
    }
    Some(rest[..=self_close + 1].to_owned())
}

#[cfg(test)]
mod tests {
    use super::{
        EMPTY_WORKSHEET_XML, OoxmlZipEntry, a1_reference, blank_worksheet_with_inherited_format,
        ensure_content_type_default, ensure_content_type_override, extract_xml_element,
        image_content_type, insert_before_close_tag, next_numeric_attribute, next_part_number,
        next_relationship_id, next_sheet_id, next_worksheet_part_name,
        remove_xml_element_by_attribute, replace_xml_attribute, upsert_hyperlink_element,
        validate_row_shapes, xml_element_by_attribute, xml_element_inner_by_index,
        xml_element_inners,
    };
    use crate::xlsx::template_xml::TemplateCellValue;
    use zip::CompressionMethod;

    fn worksheet_entry(bytes: Vec<u8>) -> OoxmlZipEntry {
        OoxmlZipEntry {
            name: "xl/worksheets/sheet1.xml".to_owned(),
            is_dir: false,
            compression: CompressionMethod::Stored,
            unix_mode: None,
            bytes,
        }
    }

    #[test]
    fn blank_worksheet_inherits_format_and_columns() {
        let entries = vec![worksheet_entry(
            br#"<worksheet><sheetFormatPr defaultRowHeight="20"/><cols><col min="1" max="1" width="30" customWidth="1"/></cols><sheetData/></worksheet>"#.to_vec(),
        )];
        let xml = String::from_utf8(blank_worksheet_with_inherited_format(&entries))
            .expect("blank worksheet XML");
        assert!(xml.contains("defaultRowHeight=\"20\""));
        assert!(xml.contains("customWidth=\"1\""));
        assert!(xml.contains("<sheetData>"));
    }

    #[test]
    fn blank_worksheet_falls_back_for_missing_invalid_or_bare_sources() {
        assert_eq!(
            blank_worksheet_with_inherited_format(&[]),
            EMPTY_WORKSHEET_XML.as_bytes()
        );
        assert_eq!(
            blank_worksheet_with_inherited_format(&[worksheet_entry(vec![0xff, 0xfe, 0x00])]),
            EMPTY_WORKSHEET_XML.as_bytes()
        );
        assert_eq!(
            blank_worksheet_with_inherited_format(&[worksheet_entry(
                br"<worksheet><sheetData/></worksheet>".to_vec(),
            )]),
            EMPTY_WORKSHEET_XML.as_bytes()
        );
    }

    #[test]
    fn package_xml_helpers_cover_success_and_error_paths() {
        assert_eq!(
            extract_xml_element(
                "<worksheet><sheetData><row/></sheetData></worksheet>",
                "sheetData",
            ),
            Some("<sheetData><row/></sheetData>".to_owned())
        );
        assert!(extract_xml_element("<sheetData><row/>", "sheetData").is_none());
        assert!(extract_xml_element("<sheetData", "sheetData").is_none());

        let error = insert_before_close_tag("<a/>", "</b>", "x").expect_err("missing tag");
        assert!(error.to_string().contains("missing </b>"));
        assert_eq!(
            insert_before_close_tag("<a></a>", "</a>", "x").expect("insert"),
            "<a>x</a>"
        );
    }

    #[test]
    fn package_identifiers_advance_above_existing_values() {
        let entries = vec![
            worksheet_entry(Vec::new()),
            OoxmlZipEntry {
                name: "xl/worksheets/sheet9.xml".to_owned(),
                is_dir: false,
                compression: CompressionMethod::Stored,
                unix_mode: None,
                bytes: Vec::new(),
            },
        ];
        assert_eq!(
            next_worksheet_part_name(&entries),
            "xl/worksheets/sheet10.xml"
        );
        assert_eq!(
            next_relationship_id(r#"<Relationship Id="rId2"/><Relationship Id="rId8"/>"#),
            "rId9"
        );
        assert_eq!(
            next_sheet_id(r#"<sheets><sheet sheetId="2"/><sheet sheetId="7"/></sheets>"#),
            8
        );
    }

    // ── a1_reference 覆盖 ──────────────────────────────────────────────────

    #[test]
    fn a1_reference_single_letter_column() {
        assert_eq!(a1_reference(0, 0), "A1");
        assert_eq!(a1_reference(0, 1), "B1");
        assert_eq!(a1_reference(0, 25), "Z1");
    }

    #[test]
    fn a1_reference_multi_letter_column() {
        assert_eq!(a1_reference(0, 26), "AA1");
        assert_eq!(a1_reference(0, 27), "AB1");
    }

    #[test]
    fn a1_reference_row_offset() {
        assert_eq!(a1_reference(4, 0), "A5");
        assert_eq!(a1_reference(99, 0), "A100");
    }

    // ── image_content_type 覆盖 ────────────────────────────────────────────

    #[test]
    fn image_content_type_png() {
        assert_eq!(image_content_type("png").unwrap(), "image/png");
    }

    #[test]
    fn image_content_type_jpg() {
        assert_eq!(image_content_type("jpg").unwrap(), "image/jpeg");
    }

    #[test]
    fn image_content_type_jpeg() {
        assert_eq!(image_content_type("jpeg").unwrap(), "image/jpeg");
    }

    #[test]
    fn image_content_type_gif() {
        assert_eq!(image_content_type("gif").unwrap(), "image/gif");
    }

    #[test]
    fn image_content_type_bmp() {
        assert_eq!(image_content_type("bmp").unwrap(), "image/bmp");
    }

    #[test]
    fn image_content_type_unsupported() {
        assert!(image_content_type("webp").is_err());
        assert!(image_content_type("svg").is_err());
    }

    // ── xml_element_inners 覆盖 ────────────────────────────────────────────

    #[test]
    fn xml_element_inners_finds_multiple_elements() {
        let xml = "<authors><author>Alice</author><author>Bob</author></authors>";
        let inners = xml_element_inners(xml, "author");
        assert_eq!(inners, vec!["Alice", "Bob"]);
    }

    #[test]
    fn xml_element_inners_returns_empty_for_missing() {
        let xml = "<authors/>";
        let inners = xml_element_inners(xml, "author");
        assert!(inners.is_empty());
    }

    #[test]
    fn xml_element_inner_by_index_returns_correct_element() {
        let xml = "<authors><author>Alice</author><author>Bob</author></authors>";
        assert_eq!(xml_element_inner_by_index(xml, "author", 0), Some("Alice"));
        assert_eq!(xml_element_inner_by_index(xml, "author", 1), Some("Bob"));
        assert_eq!(xml_element_inner_by_index(xml, "author", 2), None);
    }

    // ── xml_element_by_attribute 覆盖 ──────────────────────────────────────

    #[test]
    fn xml_element_by_attribute_finds_matching_element() {
        let xml = r#"<hyperlinks><hyperlink ref="A1" r:id="rId1"/></hyperlinks>"#;
        let result = xml_element_by_attribute(xml, "hyperlink", "ref", "A1").unwrap();
        assert!(result.is_some());
        assert!(result.unwrap().contains("rId1"));
    }

    #[test]
    fn xml_element_by_attribute_returns_none_for_no_match() {
        let xml = r#"<hyperlinks><hyperlink ref="A1" r:id="rId1"/></hyperlinks>"#;
        let result = xml_element_by_attribute(xml, "hyperlink", "ref", "B2").unwrap();
        assert!(result.is_none());
    }

    // ── remove_xml_element_by_attribute 覆盖 ───────────────────────────────

    #[test]
    fn remove_xml_element_by_attribute_removes_self_closing() {
        let xml = r#"<hyperlinks><hyperlink ref="A1" r:id="rId1"/><hyperlink ref="B2" r:id="rId2"/></hyperlinks>"#;
        let (result, removed) =
            remove_xml_element_by_attribute(xml, "hyperlink", "ref", "A1").unwrap();
        assert!(removed);
        assert!(!result.contains("rId1"));
        assert!(result.contains("rId2"));
    }

    #[test]
    fn remove_xml_element_by_attribute_returns_false_when_not_found() {
        let xml = r#"<hyperlinks><hyperlink ref="A1" r:id="rId1"/></hyperlinks>"#;
        let (result, removed) =
            remove_xml_element_by_attribute(xml, "hyperlink", "ref", "C3").unwrap();
        assert!(!removed);
        assert_eq!(result, xml);
    }

    #[test]
    fn remove_xml_element_by_attribute_removes_element_with_children() {
        let xml = "<comments><comment ref=\"A1\"><text>hello</text></comment></comments>";
        let (result, removed) =
            remove_xml_element_by_attribute(xml, "comment", "ref", "A1").unwrap();
        assert!(removed);
        assert!(!result.contains("hello"));
    }

    // ── replace_xml_attribute 覆盖 ─────────────────────────────────────────

    #[test]
    fn replace_xml_attribute_updates_value() {
        let xml = r#"<element id="old" name="test"/>"#;
        let result = replace_xml_attribute(xml, "id", "new").unwrap();
        assert!(result.contains(r#"id="new""#));
        assert!(!result.contains("old"));
    }

    #[test]
    fn replace_xml_attribute_missing_attribute() {
        let xml = r#"<element name="test"/>"#;
        let result = replace_xml_attribute(xml, "id", "new");
        assert!(result.is_err());
    }

    // ── upsert_hyperlink_element 覆盖 ──────────────────────────────────────

    #[test]
    fn upsert_hyperlink_element_into_existing_hyperlinks() {
        let xml = r#"<worksheet><hyperlinks><hyperlink ref="A1"/></hyperlinks></worksheet>"#;
        let hyperlink = r#"<hyperlink ref="B2" r:id="rId1"/>"#;
        let result = upsert_hyperlink_element(xml, hyperlink).unwrap();
        assert!(result.contains("B2"));
        assert!(result.contains("A1"));
    }

    #[test]
    fn upsert_hyperlink_element_creates_hyperlinks_section() {
        let xml = r#"<worksheet><sheetData/></worksheet>"#;
        let hyperlink = r#"<hyperlink ref="A1" r:id="rId1"/>"#;
        let result = upsert_hyperlink_element(xml, hyperlink).unwrap();
        assert!(result.contains("<hyperlinks>"));
        assert!(result.contains("A1"));
    }

    // ── validate_row_shapes 覆盖 ───────────────────────────────────────────

    #[test]
    fn validate_row_shapes_accepts_matching_shapes() {
        let rows: Vec<Vec<(usize, TemplateCellValue)>> = vec![
            vec![(0, TemplateCellValue::Number("1".into()))],
            vec![(0, TemplateCellValue::Number("2".into()))],
        ];
        let heights: Vec<Option<u16>> = vec![Some(20), Some(30)];
        let styles: Vec<Vec<Option<u32>>> = vec![vec![None], vec![None]];
        let absent: Vec<bool> = vec![false, false];
        assert!(validate_row_shapes(&rows, &heights, &styles, &absent).is_ok());
    }

    #[test]
    fn validate_row_shapes_rejects_mismatched_absent_count() {
        let rows: Vec<Vec<(usize, TemplateCellValue)>> =
            vec![vec![(0, TemplateCellValue::Number("1".into()))]];
        let absent: Vec<bool> = vec![false, true, false]; // 3 != 1
        assert!(validate_row_shapes(&rows, &[], &[], &absent).is_err());
    }

    #[test]
    fn validate_row_shapes_rejects_mismatched_height_count() {
        let rows: Vec<Vec<(usize, TemplateCellValue)>> =
            vec![vec![(0, TemplateCellValue::Number("1".into()))]];
        let heights: Vec<Option<u16>> = vec![Some(20), Some(30)]; // 2 != 1
        assert!(validate_row_shapes(&rows, &heights, &[], &[]).is_err());
    }

    #[test]
    fn validate_row_shapes_rejects_mismatched_style_count() {
        let rows: Vec<Vec<(usize, TemplateCellValue)>> =
            vec![vec![(0, TemplateCellValue::Number("1".into()))]];
        let styles: Vec<Vec<Option<u32>>> = vec![vec![None], vec![None]]; // 2 != 1
        assert!(validate_row_shapes(&rows, &[], &styles, &[]).is_err());
    }

    #[test]
    fn validate_row_shapes_accepts_empty_optional_slices() {
        let rows: Vec<Vec<(usize, TemplateCellValue)>> =
            vec![vec![(0, TemplateCellValue::Number("1".into()))]];
        assert!(validate_row_shapes(&rows, &[], &[], &[]).is_ok());
    }

    // ── next_part_number 覆盖 ──────────────────────────────────────────────

    #[test]
    fn next_part_number_starts_at_one() {
        let entries: Vec<OoxmlZipEntry> = vec![];
        assert_eq!(next_part_number(&entries, "xl/media/image", ".png"), 1);
    }

    #[test]
    fn next_part_number_advances_beyond_existing() {
        let entries = vec![OoxmlZipEntry {
            name: "xl/media/image3.png".to_owned(),
            is_dir: false,
            compression: CompressionMethod::Stored,
            unix_mode: None,
            bytes: Vec::new(),
        }];
        assert_eq!(next_part_number(&entries, "xl/media/image", ".png"), 4);
    }

    // ── next_numeric_attribute 覆盖 ────────────────────────────────────────

    #[test]
    fn next_numeric_attribute_starts_at_one() {
        assert_eq!(next_numeric_attribute("", "Id=\"rId"), 1);
    }

    #[test]
    fn next_numeric_attribute_advances_beyond_existing() {
        let xml = r#"<Relationship Id="rId3"/><Relationship Id="rId7"/>"#;
        assert_eq!(next_numeric_attribute(xml, "Id=\"rId"), 8);
    }

    // ── ensure_content_type_override 覆盖 ──────────────────────────────────

    #[test]
    fn ensure_content_type_override_adds_missing_override() {
        let mut entries = vec![OoxmlZipEntry {
            name: "[Content_Types].xml".to_owned(),
            is_dir: false,
            compression: CompressionMethod::Stored,
            unix_mode: None,
            bytes: br#"<?xml version="1.0"?><Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"></Types>"#.to_vec(),
        }];
        ensure_content_type_override(
            &mut entries,
            "/xl/comments1.xml",
            "application/vnd.openxmlformats-officedocument.spreadsheetml.comments+xml",
        )
        .unwrap();
        let xml = String::from_utf8(entries[0].bytes.clone()).unwrap();
        assert!(xml.contains("PartName=\"/xl/comments1.xml\""));
    }

    #[test]
    fn ensure_content_type_override_skips_existing() {
        let xml = br#"<?xml version="1.0"?><Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Override PartName="/xl/comments1.xml" ContentType="application/test"/></Types>"#;
        let mut entries = vec![OoxmlZipEntry {
            name: "[Content_Types].xml".to_owned(),
            is_dir: false,
            compression: CompressionMethod::Stored,
            unix_mode: None,
            bytes: xml.to_vec(),
        }];
        ensure_content_type_override(&mut entries, "/xl/comments1.xml", "application/test")
            .unwrap();
        // 不应修改内容（已存在）
        assert_eq!(entries[0].bytes, xml.to_vec());
    }

    // ── ensure_content_type_default 覆盖 ───────────────────────────────────

    #[test]
    fn ensure_content_type_default_adds_missing_default() {
        let mut entries = vec![OoxmlZipEntry {
            name: "[Content_Types].xml".to_owned(),
            is_dir: false,
            compression: CompressionMethod::Stored,
            unix_mode: None,
            bytes: br#"<?xml version="1.0"?><Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"></Types>"#.to_vec(),
        }];
        ensure_content_type_default(&mut entries, "png", "image/png").unwrap();
        let xml = String::from_utf8(entries[0].bytes.clone()).unwrap();
        assert!(xml.contains("Extension=\"png\""));
    }

    // ── extract_xml_element 覆盖 ───────────────────────────────────────────

    #[test]
    fn extract_xml_element_self_closing() {
        let result = extract_xml_element("<worksheet><cols/></worksheet>", "cols");
        assert_eq!(result, Some("<cols/>".to_owned()));
    }

    #[test]
    fn extract_xml_element_not_found() {
        let result = extract_xml_element("<worksheet/>", "missing");
        assert_eq!(result, None);
    }

    // ── 额外 import ────────────────────────────────────────────────────

    use super::super::ooxml_package::OoxmlPackage;
    use super::{
        OoxmlTemplatePackage, chart_anchor, drawing_image_relationship_id, drawing_relationship_id,
        entry_index, entry_string, image_anchor, insert_drawing_reference, relationship_target,
        relationship_target_by_type, remove_vml_comment_shape, template_column_width_pixels,
        template_row_height_pixels, vml_comment_shape, with_next_drawing_object_id,
        with_next_vml_shape_id, xml_elements,
    };

    /// 构建最小 OOXML 模板包，包含 workbook.xml、workbook.xml.rels、
    /// [Content_Types].xml、xl/styles.xml 和 xl/worksheets/sheet1.xml。
    fn minimal_template_package() -> OoxmlTemplatePackage {
        let workbook = br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<workbook xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main"
          xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">
<sheets><sheet name="Sheet1" sheetId="1" r:id="rId1"/></sheets></workbook>"#;
        let rels = br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
<Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/worksheet" Target="worksheets/sheet1.xml"/>
<Relationship Id="rId2" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles" Target="styles.xml"/>
</Relationships>"#;
        let content_types = br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
<Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
<Default Extension="xml" ContentType="application/xml"/>
<Override PartName="/xl/workbook.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.sheet.main+xml"/>
<Override PartName="/xl/worksheets/sheet1.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.worksheet+xml"/>
<Override PartName="/xl/styles.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.styles+xml"/>
</Types>"#;
        let styles = br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<styleSheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">
<fonts count="1"><font><sz val="11"/></font></fonts>
<fills count="2"><fill><patternFill patternType="none"/></fill><fill><patternFill patternType="gray125"/></fill></fills>
<borders count="1"><border/></borders>
<cellStyleXfs count="1"><xf/></cellStyleXfs>
<cellXfs count="1"><xf/></cellXfs>
</styleSheet>"#;
        let worksheet = br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main"
           xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">
<dimension ref="A1"/><sheetData></sheetData></worksheet>"#;

        let entries = vec![
            OoxmlZipEntry {
                name: "xl/workbook.xml".into(),
                is_dir: false,
                compression: CompressionMethod::Stored,
                unix_mode: None,
                bytes: workbook.to_vec(),
            },
            OoxmlZipEntry {
                name: "xl/_rels/workbook.xml.rels".into(),
                is_dir: false,
                compression: CompressionMethod::Stored,
                unix_mode: None,
                bytes: rels.to_vec(),
            },
            OoxmlZipEntry {
                name: "[Content_Types].xml".into(),
                is_dir: false,
                compression: CompressionMethod::Stored,
                unix_mode: None,
                bytes: content_types.to_vec(),
            },
            OoxmlZipEntry {
                name: "xl/styles.xml".into(),
                is_dir: false,
                compression: CompressionMethod::Stored,
                unix_mode: None,
                bytes: styles.to_vec(),
            },
            OoxmlZipEntry {
                name: "xl/worksheets/sheet1.xml".into(),
                is_dir: false,
                compression: CompressionMethod::Stored,
                unix_mode: None,
                bytes: worksheet.to_vec(),
            },
        ];
        OoxmlTemplatePackage::from_package(OoxmlPackage::from_entries(entries))
    }

    /// 带行数据的模板包
    fn template_package_with_rows() -> OoxmlTemplatePackage {
        let workbook = br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<workbook xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main"
          xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">
<sheets><sheet name="Sheet1" sheetId="1" r:id="rId1"/></sheets></workbook>"#;
        let rels = br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
<Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/worksheet" Target="worksheets/sheet1.xml"/>
<Relationship Id="rId2" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles" Target="styles.xml"/>
</Relationships>"#;
        let content_types = br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
<Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
<Default Extension="xml" ContentType="application/xml"/>
<Override PartName="/xl/workbook.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.sheet.main+xml"/>
<Override PartName="/xl/worksheets/sheet1.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.worksheet+xml"/>
<Override PartName="/xl/styles.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.styles+xml"/>
</Types>"#;
        let styles = br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<styleSheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">
<fonts count="1"><font><sz val="11"/></font></fonts>
<fills count="2"><fill><patternFill patternType="none"/></fill><fill><patternFill patternType="gray125"/></fill></fills>
<borders count="1"><border/></borders>
<cellStyleXfs count="1"><xf/></cellStyleXfs>
<cellXfs count="1"><xf/></cellXfs>
</styleSheet>"#;
        let worksheet = br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main"
           xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">
<dimension ref="A1:B3"/>
<sheetData>
<row r="1"><c r="A1" t="inlineStr"><is><t>Hello</t></is></c><c r="B1"><v>42</v></c></row>
<row r="3"><c r="A3"><v>99</v></c></row>
</sheetData></worksheet>"#;
        let entries = vec![
            OoxmlZipEntry {
                name: "xl/workbook.xml".into(),
                is_dir: false,
                compression: CompressionMethod::Stored,
                unix_mode: None,
                bytes: workbook.to_vec(),
            },
            OoxmlZipEntry {
                name: "xl/_rels/workbook.xml.rels".into(),
                is_dir: false,
                compression: CompressionMethod::Stored,
                unix_mode: None,
                bytes: rels.to_vec(),
            },
            OoxmlZipEntry {
                name: "[Content_Types].xml".into(),
                is_dir: false,
                compression: CompressionMethod::Stored,
                unix_mode: None,
                bytes: content_types.to_vec(),
            },
            OoxmlZipEntry {
                name: "xl/styles.xml".into(),
                is_dir: false,
                compression: CompressionMethod::Stored,
                unix_mode: None,
                bytes: styles.to_vec(),
            },
            OoxmlZipEntry {
                name: "xl/worksheets/sheet1.xml".into(),
                is_dir: false,
                compression: CompressionMethod::Stored,
                unix_mode: None,
                bytes: worksheet.to_vec(),
            },
        ];
        OoxmlTemplatePackage::from_package(OoxmlPackage::from_entries(entries))
    }

    // ── relationship_target 覆盖 ──────────────────────────────────────

    #[test]
    fn relationship_target_finds_by_id() {
        let xml = r#"<?xml version="1.0"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
<Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/worksheet" Target="worksheets/sheet1.xml"/>
<Relationship Id="rId2" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles" Target="styles.xml"/>
</Relationships>"#;
        let target = relationship_target(xml, "rId1").unwrap();
        assert_eq!(target, "worksheets/sheet1.xml");
        let target2 = relationship_target(xml, "rId2").unwrap();
        assert_eq!(target2, "styles.xml");
    }

    #[test]
    fn relationship_target_returns_none_for_missing() {
        let xml = r#"<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
<Relationship Id="rId1" Target="worksheets/sheet1.xml"/>
</Relationships>"#;
        assert!(relationship_target(xml, "rId99").is_none());
    }

    // ── relationship_target_by_type 覆盖 ─────────────────────────────

    #[test]
    fn relationship_target_by_type_finds_by_suffix() {
        let xml = r#"<?xml version="1.0"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
<Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/comments" Target="../comments1.xml"/>
<Relationship Id="rId2" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/vmlDrawing" Target="../drawings/vmlDrawing1.vml"/>
</Relationships>"#;
        let comments = relationship_target_by_type(xml, "/comments").unwrap();
        assert_eq!(comments, "../comments1.xml");
        let vml = relationship_target_by_type(xml, "/vmlDrawing").unwrap();
        assert_eq!(vml, "../drawings/vmlDrawing1.vml");
    }

    #[test]
    fn relationship_target_by_type_returns_none_for_missing() {
        let xml = r#"<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"></Relationships>"#;
        assert!(relationship_target_by_type(xml, "/comments").is_none());
    }

    // ── vml_comment_shape 覆盖 ────────────────────────────────────────

    #[test]
    fn vml_comment_shape_finds_matching_shape() {
        let xml = r#"<xml xmlns:v="urn:schemas-microsoft-com:vml">
<v:shape><x:Row>2</x:Row><x:Column>3</x:Column></v:shape>
<v:shape><x:Row>5</x:Row><x:Column>1</x:Column></v:shape>
</xml>"#;
        let shape = vml_comment_shape(xml, 2, 3).unwrap();
        assert!(shape.is_some());
        assert!(shape.unwrap().contains("<x:Row>2</x:Row>"));
    }

    #[test]
    fn vml_comment_shape_returns_none_for_no_match() {
        let xml = r#"<xml><v:shape><x:Row>2</x:Row><x:Column>3</x:Column></v:shape></xml>"#;
        let shape = vml_comment_shape(xml, 99, 99).unwrap();
        assert!(shape.is_none());
    }

    #[test]
    fn vml_comment_shape_matches_with_short_row_column_tags() {
        let xml = r#"<xml><v:shape><Row>1</Row><Column>0</Column></v:shape></xml>"#;
        let shape = vml_comment_shape(xml, 1, 0).unwrap();
        assert!(shape.is_some());
    }

    #[test]
    fn vml_comment_shape_error_for_unterminated() {
        let xml = r#"<xml><v:shape><x:Row>1</x:Row>"#;
        let result = vml_comment_shape(xml, 1, 0);
        assert!(result.is_err());
    }

    // ── with_next_vml_shape_id 覆盖 ───────────────────────────────────

    #[test]
    fn with_next_vml_shape_id_assigns_new_id() {
        let target_vml = r#"<xml><v:shape id="_x0000_s1026"><x:Row>1</x:Row></v:shape></xml>"#;
        let source_shape = r#"<v:shape id="_x0000_s1026"><x:Row>2</x:Row></v:shape>"#;
        let result = with_next_vml_shape_id(target_vml, source_shape).unwrap();
        assert!(result.contains("id=\"_x0000_s1027\""));
    }

    #[test]
    fn with_next_vml_shape_id_starts_at_one_for_empty() {
        let target_vml = "<xml/>";
        let source_shape = r#"<v:shape id="_x0000_s1026"><x:Row>1</x:Row></v:shape>"#;
        let result = with_next_vml_shape_id(target_vml, source_shape).unwrap();
        assert!(result.contains("id=\"_x0000_s1\""));
    }

    #[test]
    fn with_next_vml_shape_id_error_for_no_id() {
        let target_vml = "<xml/>";
        let source_shape = "<v:shape><x:Row>1</x:Row></v:shape>";
        let result = with_next_vml_shape_id(target_vml, source_shape);
        assert!(result.is_err());
    }

    #[test]
    fn with_next_vml_shape_id_error_for_no_closing_bracket() {
        let target_vml = "<xml/>";
        let source_shape = "<v:shape";
        let result = with_next_vml_shape_id(target_vml, source_shape);
        assert!(result.is_err());
    }

    // ── remove_vml_comment_shape 覆盖 ─────────────────────────────────

    #[test]
    fn remove_vml_comment_shape_removes_matching() {
        let xml = r#"<xml>
<v:shape><x:Row>1</x:Row><x:Column>2</x:Column>data</v:shape>
<v:shape><x:Row>3</x:Row><x:Column>4</x:Column>keep</v:shape>
</xml>"#;
        let (result, removed) = remove_vml_comment_shape(xml, 1, 2).unwrap();
        assert!(removed);
        assert!(!result.contains("data"));
        assert!(result.contains("keep"));
    }

    #[test]
    fn remove_vml_comment_shape_returns_false_for_no_match() {
        let xml = r#"<xml><v:shape><x:Row>1</x:Row><x:Column>2</x:Column></v:shape></xml>"#;
        let (result, removed) = remove_vml_comment_shape(xml, 99, 99).unwrap();
        assert!(!removed);
        assert_eq!(result, xml);
    }

    #[test]
    fn remove_vml_comment_shape_short_tags() {
        let xml = r#"<xml><v:shape><Row>5</Row><Column>0</Column></v:shape></xml>"#;
        let (_, removed) = remove_vml_comment_shape(xml, 5, 0).unwrap();
        assert!(removed);
    }

    #[test]
    fn remove_vml_comment_shape_error_for_unterminated() {
        let xml = r#"<xml><v:shape><x:Row>1</x:Row>"#;
        let result = remove_vml_comment_shape(xml, 1, 0);
        assert!(result.is_err());
    }

    // ── chart_anchor 覆盖 ─────────────────────────────────────────────

    #[test]
    fn chart_anchor_finds_two_cell_anchor() {
        let xml = r#"<xdr:wsDr xmlns:xdr="...">
<xdr:twoCellAnchor><xdr:graphicFrame><c:chart r:id="rId1"/></xdr:graphicFrame></xdr:twoCellAnchor>
</xdr:wsDr>"#;
        let anchor = chart_anchor(xml).unwrap();
        assert!(anchor.contains("twoCellAnchor"));
        assert!(anchor.contains("rId1"));
    }

    #[test]
    fn chart_anchor_finds_one_cell_anchor() {
        let xml = r#"<xdr:wsDr>
<xdr:oneCellAnchor><xdr:graphicFrame><c:chart r:id="rId2"/></xdr:graphicFrame></xdr:oneCellAnchor>
</xdr:wsDr>"#;
        let anchor = chart_anchor(xml).unwrap();
        assert!(anchor.contains("oneCellAnchor"));
    }

    #[test]
    fn chart_anchor_finds_absolute_anchor() {
        let xml = r#"<xdr:wsDr>
<xdr:absoluteAnchor><xdr:graphicFrame><c:chart r:id="rId3"/></xdr:graphicFrame></xdr:absoluteAnchor>
</xdr:wsDr>"#;
        let anchor = chart_anchor(xml).unwrap();
        assert!(anchor.contains("absoluteAnchor"));
    }

    #[test]
    fn chart_anchor_returns_none_for_empty() {
        assert!(chart_anchor("<xdr:wsDr/>").is_none());
    }

    // ── image_anchor 覆盖 ─────────────────────────────────────────────

    #[test]
    fn image_anchor_finds_pic_anchor() {
        let xml = r#"<xdr:wsDr>
<xdr:twoCellAnchor><xdr:pic><xdr:nvPicPr/></xdr:pic></xdr:twoCellAnchor>
</xdr:wsDr>"#;
        let anchor = image_anchor(xml).unwrap();
        assert!(anchor.contains("xdr:pic"));
    }

    #[test]
    fn image_anchor_finds_short_pic_tag() {
        let xml = r#"<xdr:wsDr>
<xdr:twoCellAnchor><pic><nvPicPr/></pic></xdr:twoCellAnchor>
</xdr:wsDr>"#;
        let anchor = image_anchor(xml).unwrap();
        assert!(anchor.contains("<pic"));
    }

    #[test]
    fn image_anchor_skips_non_pic_anchors() {
        let xml = r#"<xdr:wsDr>
<xdr:twoCellAnchor><xdr:graphicFrame/></xdr:twoCellAnchor>
<xdr:oneCellAnchor><xdr:pic><xdr:nvPicPr/></xdr:pic></xdr:oneCellAnchor>
</xdr:wsDr>"#;
        let anchor = image_anchor(xml).unwrap();
        assert!(anchor.contains("oneCellAnchor"));
    }

    #[test]
    fn image_anchor_returns_none_for_empty() {
        assert!(image_anchor("<xdr:wsDr/>").is_none());
    }

    // ── drawing_image_relationship_id 覆盖 ────────────────────────────

    #[test]
    fn drawing_image_relationship_id_finds_a_blip() {
        let xml = r#"<xdr:pic><xdr:nvPicPr/><xdr:blipFill><a:blip r:embed="rId1"/></xdr:blipFill></xdr:pic>"#;
        let id = drawing_image_relationship_id(xml).unwrap();
        assert_eq!(id, "rId1");
    }

    #[test]
    fn drawing_image_relationship_id_finds_short_blip() {
        // 注意：xml_elements 对 "blip" 也会匹配 <blipFill>，因此直接用 <blip> 元素
        let xml = r#"<pic><blip r:embed="rId2"/></pic>"#;
        let id = drawing_image_relationship_id(xml).unwrap();
        assert_eq!(id, "rId2");
    }

    #[test]
    fn drawing_image_relationship_id_returns_none_for_missing() {
        let xml = "<pic/>";
        assert!(drawing_image_relationship_id(xml).is_none());
    }

    // ── with_next_drawing_object_id 覆盖 ──────────────────────────────

    #[test]
    fn with_next_drawing_object_id_assigns_next_id() {
        let target = r#"<xdr:wsDr><xdr:twoCellAnchor><xdr:pic><xdr:nvPicPr><xdr:cNvPr id="3" name="Picture 3"/></xdr:nvPicPr></xdr:pic></xdr:twoCellAnchor></xdr:wsDr>"#;
        let source = r#"<xdr:twoCellAnchor><xdr:pic><xdr:nvPicPr><xdr:cNvPr id="1" name="Picture 1"/></xdr:nvPicPr></xdr:pic></xdr:twoCellAnchor>"#;
        let result = with_next_drawing_object_id(target, source).unwrap();
        assert!(result.contains("id=\"4\""));
        assert!(result.contains("name=\"Picture 4\""));
    }

    #[test]
    fn with_next_drawing_object_id_error_for_no_cnvp() {
        let target = "<xdr:wsDr/>";
        let source = "<xdr:twoCellAnchor/>";
        let result = with_next_drawing_object_id(target, source);
        assert!(result.is_err());
    }

    // ── template_column_width_pixels 覆盖 ─────────────────────────────

    #[test]
    fn template_column_width_pixels_uses_col_definition() {
        let xml = r#"<worksheet><cols><col min="1" max="5" width="20"/></cols></worksheet>"#;
        // column 0 => one_based 1, width 20 => (20*7+5).round() = 145
        assert_eq!(template_column_width_pixels(xml, 0), 145);
    }

    #[test]
    fn template_column_width_pixels_falls_back_to_default() {
        let xml = r#"<worksheet><sheetFormatPr defaultColWidth="10"/></worksheet>"#;
        // width 10 => (10*7+5).round() = 75
        assert_eq!(template_column_width_pixels(xml, 0), 75);
    }

    #[test]
    fn template_column_width_pixels_default_64() {
        let xml = "<worksheet/>";
        assert_eq!(template_column_width_pixels(xml, 0), 64);
    }

    #[test]
    fn template_column_width_pixels_filters_non_positive() {
        let xml = r#"<worksheet><cols><col min="1" max="5" width="-1"/></cols></worksheet>"#;
        // width -1 is not finite positive, falls back to 64
        assert_eq!(template_column_width_pixels(xml, 0), 64);
    }

    // ── template_row_height_pixels 覆盖 ───────────────────────────────

    #[test]
    fn template_row_height_pixels_uses_row_ht() {
        let xml = r#"<worksheet><sheetData><row r="1" ht="30"/></sheetData></worksheet>"#;
        // row 0 => one_based 1, ht 30 => (30*4/3).round() = 40
        assert_eq!(template_row_height_pixels(xml, 0), 40);
    }

    #[test]
    fn template_row_height_pixels_falls_back_to_default() {
        let xml = r#"<worksheet><sheetFormatPr defaultRowHeight="15"/></worksheet>"#;
        // height 15 => (15*4/3).round() = 20
        assert_eq!(template_row_height_pixels(xml, 0), 20);
    }

    #[test]
    fn template_row_height_pixels_default_20() {
        let xml = "<worksheet/>";
        assert_eq!(template_row_height_pixels(xml, 0), 20);
    }

    // ── drawing_relationship_id 覆盖 ──────────────────────────────────

    #[test]
    fn drawing_relationship_id_finds_drawing_element() {
        let xml = r#"<worksheet><drawing r:id="rId2"/></worksheet>"#;
        let id = drawing_relationship_id(xml).unwrap();
        assert_eq!(id, "rId2");
    }

    #[test]
    fn drawing_relationship_id_returns_none_for_missing() {
        let xml = "<worksheet/>";
        assert!(drawing_relationship_id(xml).is_none());
    }

    // ── insert_drawing_reference 覆盖 ─────────────────────────────────

    #[test]
    fn insert_drawing_reference_inserts_before_worksheet_end() {
        let xml = "<worksheet><sheetData/></worksheet>";
        let result = insert_drawing_reference(xml, "rId1").unwrap();
        assert!(result.contains("<drawing r:id=\"rId1\"/>"));
        assert!(result.contains("</worksheet>"));
    }

    #[test]
    fn insert_drawing_reference_inserts_before_legacy_drawing() {
        let xml = "<worksheet><sheetData/><legacyDrawing r:id=\"rId3\"/></worksheet>";
        let result = insert_drawing_reference(xml, "rId1").unwrap();
        let drawing_pos = result.find("<drawing").unwrap();
        let legacy_pos = result.find("<legacyDrawing").unwrap();
        assert!(drawing_pos < legacy_pos);
    }

    #[test]
    fn insert_drawing_reference_error_for_missing_end() {
        let result = insert_drawing_reference("<worksheet>", "rId1");
        assert!(result.is_err());
    }

    // ── upsert_hyperlink_element 额外路径 ─────────────────────────────

    #[test]
    fn upsert_hyperlink_element_into_self_closing_hyperlinks() {
        let xml = r#"<worksheet><sheetData/><hyperlinks/></worksheet>"#;
        let hyperlink = r#"<hyperlink ref="A1" r:id="rId1"/>"#;
        let result = upsert_hyperlink_element(xml, hyperlink).unwrap();
        assert!(result.contains("<hyperlinks>"));
        assert!(result.contains("</hyperlinks>"));
        assert!(result.contains("A1"));
    }

    #[test]
    fn upsert_hyperlink_element_creates_before_print_options() {
        let xml = r#"<worksheet><sheetData/><printOptions/></worksheet>"#;
        let hyperlink = r#"<hyperlink ref="A1" r:id="rId1"/>"#;
        let result = upsert_hyperlink_element(xml, hyperlink).unwrap();
        let hyper_pos = result.find("<hyperlinks>").unwrap();
        let print_pos = result.find("<printOptions").unwrap();
        assert!(hyper_pos < print_pos);
    }

    #[test]
    fn upsert_hyperlink_element_creates_before_drawing() {
        let xml = r#"<worksheet><sheetData/><drawing r:id="rId2"/></worksheet>"#;
        let hyperlink = r#"<hyperlink ref="A1" r:id="rId1"/>"#;
        let result = upsert_hyperlink_element(xml, hyperlink).unwrap();
        assert!(result.contains("<hyperlinks>"));
    }

    // ── validate_row_shapes 额外路径 ──────────────────────────────────

    #[test]
    fn validate_row_shapes_rejects_cell_styles_inner_mismatch() {
        let rows: Vec<Vec<(usize, TemplateCellValue)>> = vec![
            vec![(0, TemplateCellValue::Number("1".into()))],
            vec![
                (0, TemplateCellValue::Number("2".into())),
                (1, TemplateCellValue::Number("3".into())),
            ],
        ];
        let styles: Vec<Vec<Option<u32>>> = vec![vec![None], vec![None]]; // inner len 1 != 2
        assert!(validate_row_shapes(&rows, &[], &styles, &[]).is_err());
    }

    // ── xml_element_by_attribute 额外路径 ─────────────────────────────

    #[test]
    fn xml_element_by_attribute_finds_closed_element() {
        let xml = r#"<comments><comment ref="A1"><text>hello</text></comment></comments>"#;
        let result = xml_element_by_attribute(xml, "comment", "ref", "A1").unwrap();
        assert!(result.is_some());
        assert!(result.unwrap().contains("hello"));
    }

    #[test]
    fn xml_element_by_attribute_error_for_unterminated() {
        let xml = r#"<comments><comment ref="A1""#;
        let result = xml_element_by_attribute(xml, "comment", "ref", "A1");
        assert!(result.is_err());
    }

    #[test]
    fn xml_element_by_attribute_skips_tag_name_prefix() {
        // e.g., <hyperlinkExtra> should not match <hyperlink>
        let xml = r#"<hyperlinkExtra ref="A1"/><hyperlink ref="B2" r:id="rId1"/>"#;
        let result = xml_element_by_attribute(xml, "hyperlink", "ref", "B2").unwrap();
        assert!(result.is_some());
    }

    // ── remove_xml_element_by_attribute 额外路径 ──────────────────────

    #[test]
    fn remove_xml_element_by_attribute_error_for_unterminated() {
        let xml = r#"<comments><comment ref="A1""#;
        let result = remove_xml_element_by_attribute(xml, "comment", "ref", "A1");
        assert!(result.is_err());
    }

    #[test]
    fn remove_xml_element_by_attribute_error_for_missing_close() {
        let xml = r#"<comments><comment ref="A1"><text>hello</text>"#;
        let result = remove_xml_element_by_attribute(xml, "comment", "ref", "A1");
        assert!(result.is_err());
    }

    // ── replace_xml_attribute 额外路径 ────────────────────────────────

    #[test]
    fn replace_xml_attribute_error_for_no_closing_bracket() {
        let xml = "<element id=\"old\"";
        let result = replace_xml_attribute(xml, "id", "new");
        assert!(result.is_err());
    }

    #[test]
    fn replace_xml_attribute_error_for_unterminated_value() {
        let xml = "<element id=\"unclosed";
        let result = replace_xml_attribute(xml, "id", "new");
        assert!(result.is_err());
    }

    // ── xml_elements 覆盖 ─────────────────────────────────────────────

    #[test]
    fn xml_elements_finds_multiple() {
        // 注意：xml_elements 通过前缀匹配，<sheets> 也会匹配 <sheet
        let xml = r#"<root><item name="A"/><item name="B"/></root>"#;
        let items: Vec<&str> = xml_elements(xml, "item").collect();
        assert_eq!(items.len(), 2);
    }

    // ── entry_index 覆盖 ──────────────────────────────────────────────

    #[test]
    fn entry_index_finds_existing() {
        let entries = vec![OoxmlZipEntry {
            name: "xl/workbook.xml".into(),
            is_dir: false,
            compression: CompressionMethod::Stored,
            unix_mode: None,
            bytes: Vec::new(),
        }];
        assert_eq!(entry_index(&entries, "xl/workbook.xml").unwrap(), 0);
    }

    #[test]
    fn entry_index_error_for_missing() {
        let result = entry_index(&[], "missing.xml");
        assert!(result.is_err());
    }

    // ── entry_string 覆盖 ─────────────────────────────────────────────

    #[test]
    fn entry_string_returns_utf8_content() {
        let entry = OoxmlZipEntry {
            name: "test.xml".into(),
            is_dir: false,
            compression: CompressionMethod::Stored,
            unix_mode: None,
            bytes: b"<test/>".to_vec(),
        };
        assert_eq!(entry_string(&entry).unwrap(), "<test/>");
    }

    #[test]
    fn entry_string_error_for_invalid_utf8() {
        let entry = OoxmlZipEntry {
            name: "test.xml".into(),
            is_dir: false,
            compression: CompressionMethod::Stored,
            unix_mode: None,
            bytes: vec![0xff, 0xfe],
        };
        assert!(entry_string(&entry).is_err());
    }

    // ═══════════════════════════════════════════════════════════════════
    // OoxmlTemplatePackage 结构体方法覆盖
    // ═══════════════════════════════════════════════════════════════════

    #[test]
    fn from_package_round_trips() {
        let pkg = minimal_template_package();
        // into_package 取回底层包
        let inner = pkg.into_package();
        let pkg2 = OoxmlTemplatePackage::from_package(inner);
        let names = pkg2.sheet_names().unwrap();
        assert_eq!(names, vec!["Sheet1"]);
    }

    #[test]
    fn sheet_names_returns_ordered_names() {
        // 构建含两个工作表的模板
        let mut pkg = minimal_template_package();
        pkg.create_sheet("第二页").unwrap();
        let names = pkg.sheet_names().unwrap();
        assert_eq!(names, vec!["Sheet1", "第二页"]);
    }

    #[test]
    fn next_row_for_sheet_empty_returns_zero() {
        let pkg = minimal_template_package();
        assert_eq!(pkg.next_row_for_sheet("Sheet1").unwrap(), 0);
    }

    #[test]
    fn next_row_for_sheet_with_data_returns_next() {
        let pkg = template_package_with_rows();
        // 行 1 和行 3 存在，最大行号 3，返回 4
        assert_eq!(pkg.next_row_for_sheet("Sheet1").unwrap(), 4);
    }

    #[test]
    fn next_row_for_sheet_error_for_missing_sheet() {
        let pkg = minimal_template_package();
        assert!(pkg.next_row_for_sheet("不存在").is_err());
    }

    #[test]
    fn worksheet_path_by_name_finds_sheet() {
        let pkg = minimal_template_package();
        let path = pkg.worksheet_path_by_name("Sheet1").unwrap();
        assert!(path.contains("sheet1"));
    }

    #[test]
    fn worksheet_path_by_name_error_for_missing() {
        let pkg = minimal_template_package();
        assert!(pkg.worksheet_path_by_name("Missing").is_err());
    }

    #[test]
    fn worksheet_path_by_index_works() {
        let pkg = minimal_template_package();
        let (name, path) = pkg.worksheet_path_by_index(0).unwrap();
        assert_eq!(name, "Sheet1");
        assert!(path.contains("sheet1"));
    }

    #[test]
    fn worksheet_path_by_index_error_for_out_of_range() {
        let pkg = minimal_template_package();
        assert!(pkg.worksheet_path_by_index(99).is_err());
    }

    #[test]
    fn sheet_name_by_worksheet_path_resolves() {
        let pkg = minimal_template_package();
        let path = pkg.worksheet_path_by_name("Sheet1").unwrap();
        let name = pkg.sheet_name_by_worksheet_path(&path).unwrap();
        assert_eq!(name, "Sheet1");
    }

    #[test]
    fn sheet_name_by_worksheet_path_error_for_missing() {
        let pkg = minimal_template_package();
        assert!(
            pkg.sheet_name_by_worksheet_path("xl/worksheets/sheet99.xml")
                .is_err()
        );
    }

    // ── ensure_sheet 覆盖 ─────────────────────────────────────────────

    #[test]
    fn ensure_sheet_noop_if_exists() {
        let mut pkg = minimal_template_package();
        pkg.ensure_sheet("Sheet1").unwrap();
        assert_eq!(pkg.sheet_names().unwrap().len(), 1);
    }

    #[test]
    fn ensure_sheet_creates_new() {
        let mut pkg = minimal_template_package();
        pkg.ensure_sheet("NewSheet").unwrap();
        assert_eq!(pkg.sheet_names().unwrap(), vec!["Sheet1", "NewSheet"]);
    }

    // ── create_sheet 覆盖 ─────────────────────────────────────────────

    #[test]
    fn create_sheet_adds_workbook_and_content_types() {
        let mut pkg = minimal_template_package();
        pkg.create_sheet("Sheet2").unwrap();
        let names = pkg.sheet_names().unwrap();
        assert_eq!(names.len(), 2);
        assert_eq!(names[1], "Sheet2");
        // 验证可以获取路径
        let path = pkg.worksheet_path_by_name("Sheet2").unwrap();
        assert!(path.contains("sheet"));
    }

    #[test]
    fn create_sheet_with_special_characters_in_name() {
        let mut pkg = minimal_template_package();
        pkg.create_sheet("A & B < C").unwrap();
        let names = pkg.sheet_names().unwrap();
        assert_eq!(names.len(), 2);
        // 属性值包含转义后的 XML 实体
        assert!(names[1].contains("A"));
    }

    // ── append_rows 覆盖 ──────────────────────────────────────────────

    #[test]
    fn append_rows_empty_returns_next_row() {
        let mut pkg = template_package_with_rows();
        let next = pkg.append_rows("Sheet1", &[], &[], &[], &[]).unwrap();
        assert_eq!(next, 4);
    }

    #[test]
    fn append_rows_adds_data() {
        let mut pkg = minimal_template_package();
        let rows = vec![
            vec![(0, TemplateCellValue::Text("hello".into()))],
            vec![(1, TemplateCellValue::Number("42".into()))],
        ];
        let next = pkg.append_rows("Sheet1", &rows, &[], &[], &[]).unwrap();
        assert_eq!(next, 3);
        // 验证下一行为 3
        let next2 = pkg.next_row_for_sheet("Sheet1").unwrap();
        assert_eq!(next2, 3);
    }

    #[test]
    fn append_rows_with_heights_and_styles() {
        let mut pkg = minimal_template_package();
        let rows = vec![vec![(0, TemplateCellValue::Number("1".into()))]];
        let heights = vec![Some(30)];
        let styles = vec![vec![Some(1)]];
        pkg.append_rows("Sheet1", &rows, &heights, &styles, &[])
            .unwrap();
    }

    #[test]
    fn append_rows_with_absent_rows() {
        let mut pkg = minimal_template_package();
        let rows = vec![
            vec![(0, TemplateCellValue::Number("1".into()))],
            vec![(0, TemplateCellValue::Number("2".into()))],
        ];
        let absent = vec![false, true]; // 第二行缺席
        let next = pkg.append_rows("Sheet1", &rows, &[], &[], &absent).unwrap();
        assert_eq!(next, 3); // 只追加了一行
    }

    #[test]
    fn append_rows_error_for_shape_mismatch() {
        let mut pkg = minimal_template_package();
        let rows = vec![vec![(0, TemplateCellValue::Number("1".into()))]];
        let heights = vec![Some(20), Some(30)]; // 长度不匹配
        let result = pkg.append_rows("Sheet1", &rows, &heights, &[], &[]);
        assert!(result.is_err());
    }

    // ── apply_sheet_layout 覆盖 ───────────────────────────────────────

    #[test]
    fn apply_sheet_layout_noop_if_empty() {
        let mut pkg = minimal_template_package();
        pkg.apply_sheet_layout("Sheet1", &[], &[]).unwrap();
    }

    #[test]
    fn apply_sheet_layout_sets_column_widths() {
        let mut pkg = minimal_template_package();
        pkg.apply_sheet_layout("Sheet1", &[(0, 20), (1, 30)], &[])
            .unwrap();
    }

    #[test]
    fn apply_sheet_layout_sets_merge_ranges() {
        use super::super::template_xml::TemplateMergeRange;
        let mut pkg = minimal_template_package();
        let ranges = vec![TemplateMergeRange {
            first_row: 0,
            first_column: 0,
            last_row: 1,
            last_column: 1,
        }];
        pkg.apply_sheet_layout("Sheet1", &[], &ranges).unwrap();
    }

    // ── set_cell 覆盖 ─────────────────────────────────────────────────

    #[test]
    fn set_cell_updates_value() {
        let mut pkg = template_package_with_rows();
        pkg.set_cell("Sheet1", 0, 0, &TemplateCellValue::Text("updated".into()))
            .unwrap();
    }

    #[test]
    fn set_cell_error_for_missing_sheet() {
        let mut pkg = minimal_template_package();
        let result = pkg.set_cell("Missing", 0, 0, &TemplateCellValue::Empty);
        assert!(result.is_err());
    }

    // ── protect_sheet 覆盖 ────────────────────────────────────────────

    #[test]
    fn protect_sheet_adds_protection() {
        let mut pkg = minimal_template_package();
        pkg.protect_sheet("Sheet1", "password123").unwrap();
        let path = pkg.worksheet_path_by_name("Sheet1").unwrap();
        let xml = pkg.entry_xml(&path).unwrap();
        assert!(xml.contains("sheetProtection"));
    }

    // ── import_compiled_styles 覆盖 ───────────────────────────────────

    #[test]
    fn import_compiled_styles_empty_returns_empty() {
        let mut pkg = minimal_template_package();
        let result = pkg.import_compiled_styles(&[], 0).unwrap();
        assert!(result.is_empty());
    }

    // ── to_bytes / save_to_writer 覆盖 ────────────────────────────────

    #[test]
    fn to_bytes_produces_valid_zip() {
        let pkg = minimal_template_package();
        let bytes = pkg.to_bytes().unwrap();
        assert!(!bytes.is_empty());
        // 验证可以重新加载
        let reloaded = OoxmlTemplatePackage::from_bytes(&bytes).unwrap();
        assert_eq!(reloaded.sheet_names().unwrap(), vec!["Sheet1"]);
    }

    #[test]
    fn save_to_writer_writes_bytes() {
        let pkg = minimal_template_package();
        let mut buf = Vec::new();
        pkg.save_to_writer(&mut buf).unwrap();
        assert!(!buf.is_empty());
    }

    // ── from_bytes 覆盖 ───────────────────────────────────────────────

    #[test]
    fn from_bytes_round_trips() {
        let pkg = minimal_template_package();
        let bytes = pkg.to_bytes().unwrap();
        let reloaded = OoxmlTemplatePackage::from_bytes(&bytes).unwrap();
        assert_eq!(reloaded.sheet_names().unwrap(), vec!["Sheet1"]);
    }

    #[test]
    fn from_bytes_error_for_invalid_zip() {
        let result = OoxmlTemplatePackage::from_bytes(&[0xff, 0xfe, 0x00]);
        assert!(result.is_err());
    }

    // ── Deref / DerefMut 覆盖 ─────────────────────────────────────────

    #[test]
    fn deref_allows_entry_access() {
        let pkg = minimal_template_package();
        // 通过 Deref 访问 OoxmlPackage 的方法
        let inner = &*pkg;
        assert!(!inner.is_empty());
    }

    #[test]
    fn deref_mut_allows_entry_modification() {
        let mut pkg = minimal_template_package();
        let inner = &mut *pkg;
        let count = inner.len();
        inner.push(OoxmlZipEntry {
            name: "test.txt".into(),
            is_dir: false,
            compression: CompressionMethod::Stored,
            unix_mode: None,
            bytes: b"test".to_vec(),
        });
        assert_eq!(inner.len(), count + 1);
    }

    // ── 多工作表场景覆盖 ──────────────────────────────────────────────

    #[test]
    fn multi_sheet_create_and_append() {
        let mut pkg = minimal_template_package();
        pkg.create_sheet("数据").unwrap();
        pkg.create_sheet("汇总").unwrap();
        assert_eq!(pkg.sheet_names().unwrap().len(), 3);

        // 在每个表追加数据
        let rows = vec![vec![(0, TemplateCellValue::Number("100".into()))]];
        pkg.append_rows("Sheet1", &rows, &[], &[], &[]).unwrap();
        pkg.append_rows("数据", &rows, &[], &[], &[]).unwrap();
        pkg.append_rows("汇总", &rows, &[], &[], &[]).unwrap();

        // 验证每张表都有数据
        assert_eq!(pkg.next_row_for_sheet("Sheet1").unwrap(), 2);
        assert_eq!(pkg.next_row_for_sheet("数据").unwrap(), 2);
        assert_eq!(pkg.next_row_for_sheet("汇总").unwrap(), 2);
    }

    // ── set_template_hyperlink 覆盖 ───────────────────────────────────

    use super::super::template_fill::{TemplateHyperlink, TemplateHyperlinkType};

    #[test]
    fn set_template_hyperlink_url_type() {
        let mut pkg = minimal_template_package();
        let hyperlink = TemplateHyperlink::new("https://example.com", TemplateHyperlinkType::Url);
        pkg.set_template_hyperlink("Sheet1", 0, 0, &hyperlink)
            .unwrap();
    }

    #[test]
    fn set_template_hyperlink_email_type() {
        let mut pkg = minimal_template_package();
        let hyperlink = TemplateHyperlink::new("test@example.com", TemplateHyperlinkType::Email);
        pkg.set_template_hyperlink("Sheet1", 0, 0, &hyperlink)
            .unwrap();
    }

    #[test]
    fn set_template_hyperlink_file_type() {
        let mut pkg = minimal_template_package();
        let hyperlink = TemplateHyperlink::new("/path/to/file.xlsx", TemplateHyperlinkType::File);
        pkg.set_template_hyperlink("Sheet1", 0, 0, &hyperlink)
            .unwrap();
    }

    #[test]
    fn set_template_hyperlink_document_type() {
        let mut pkg = minimal_template_package();
        let hyperlink = TemplateHyperlink::new("Sheet2!A1", TemplateHyperlinkType::Document);
        pkg.set_template_hyperlink("Sheet1", 0, 0, &hyperlink)
            .unwrap();
    }

    #[test]
    fn set_template_hyperlink_replaces_existing() {
        let mut pkg = minimal_template_package();
        let hyperlink = TemplateHyperlink::new("https://first.com", TemplateHyperlinkType::Url);
        pkg.set_template_hyperlink("Sheet1", 0, 0, &hyperlink)
            .unwrap();
        let hyperlink2 = TemplateHyperlink::new("https://second.com", TemplateHyperlinkType::Url);
        pkg.set_template_hyperlink("Sheet1", 0, 0, &hyperlink2)
            .unwrap();
        // 验证 second.com 关系存在（在 rels 文件中）
        let rels_path = "xl/worksheets/_rels/sheet1.xml.rels";
        let rels_xml = pkg.entry_xml(rels_path).unwrap();
        assert!(rels_xml.contains("second.com"));
    }

    #[test]
    fn set_template_hyperlink_rollback_on_error() {
        let mut pkg = minimal_template_package();
        let hyperlink = TemplateHyperlink::new("https://example.com", TemplateHyperlinkType::Url);
        let result = pkg.set_template_hyperlink("不存在", 0, 0, &hyperlink);
        assert!(result.is_err());
        // 包应保持原状
        assert_eq!(pkg.sheet_names().unwrap(), vec!["Sheet1"]);
    }

    // ── import_chart 覆盖（错误路径）───────────────────────────────────

    #[test]
    fn import_chart_error_for_missing_drawing() {
        let mut pkg = minimal_template_package();
        // 编译的 XLSX 中没有 drawing 关系
        let result = pkg.import_chart(&[], "Sheet1");
        assert!(result.is_err());
    }

    // ── set_template_comment 覆盖 ─────────────────────────────────────

    use super::super::template_fill::TemplateComment;

    #[test]
    fn set_template_comment_basic() {
        let mut pkg = minimal_template_package();
        let comment = TemplateComment {
            text: "测试批注".into(),
            author: Some("作者".into()),
            visible: Some(true),
            movement: Some(0),
        };
        pkg.set_template_comment("Sheet1", 0, 0, &comment).unwrap();
    }

    // ── import_compiled_styles_onto 覆盖 ──────────────────────────────

    #[test]
    fn import_compiled_styles_onto_empty_returns_empty() {
        let mut pkg = minimal_template_package();
        let result = pkg.import_compiled_styles_onto(&[], &[]).unwrap();
        assert!(result.is_empty());
    }

    // ── remove_comment 覆盖 ───────────────────────────────────────────

    #[test]
    fn remove_comment_returns_false_for_no_comments() {
        let mut pkg = minimal_template_package();
        let result = pkg.remove_comment("Sheet1", 0, 0).unwrap();
        assert!(!result);
    }

    // ── save_to_path 覆盖 ─────────────────────────────────────────────

    #[test]
    fn save_to_path_writes_file() {
        let pkg = minimal_template_package();
        let dir = std::env::temp_dir().join("easyexcel_test_template_package");
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("test_output.xlsx");
        pkg.save_to_path(&path).unwrap();
        assert!(path.exists());
        let bytes = std::fs::read(&path).unwrap();
        assert!(!bytes.is_empty());
        let _ = std::fs::remove_file(&path);
    }

    // ── import_image 覆盖 ─────────────────────────────────────────────

    #[test]
    fn import_image_rollback_on_error() {
        let mut pkg = minimal_template_package();
        let image = super::super::template_fill::TemplateImage::new(vec![0x89, 0x50, 0x4E, 0x47]);
        // 编译一个空 XLSX 应该失败（没有图片），触发回滚
        let result = pkg.set_template_image("Sheet1", 0, 0, &image);
        assert!(result.is_err());
        // 包应保持原状
        assert_eq!(pkg.sheet_names().unwrap(), vec!["Sheet1"]);
    }
}
