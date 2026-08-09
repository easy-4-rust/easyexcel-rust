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
use super::template_styles::{merge_compiled_styles, merge_compiled_styles_onto};
use super::template_fill::{
    TemplateComment, TemplateHyperlink, TemplateHyperlinkType, TemplateImage,
    TemplateImageMovement,
};
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

        if let Some(vml_target) =
            relationship_target_by_type(&relationships_xml, "/vmlDrawing")
        {
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
        if let Err(error) = self.import_comment_inner(
            compiled_xlsx,
            sheet_name,
            row_index,
            column_index,
        ) {
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
                let location = hyperlink
                    .hyperlink_type
                    .package_target(&hyperlink.address);
                format!(
                    "<hyperlink ref=\"{}\" location=\"{}\"/>",
                    escape_xml(&reference),
                    escape_xml(&location)
                )
            }
            TemplateHyperlinkType::Url
            | TemplateHyperlinkType::Email
            | TemplateHyperlinkType::File => {
                let target = hyperlink
                    .hyperlink_type
                    .package_target(&hyperlink.address);
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
                relationships_xml = insert_before_close_tag(
                    &relationships_xml,
                    "</Relationships>",
                    &relationship,
                )?;
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
        if let Err(error) = self.set_template_image_inner(
            sheet_name,
            row_index,
            column_index,
            image,
        ) {
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
            .ok_or_else(|| Error::Xlsx("compiled image drawing relationship is missing".to_owned()))?;
        let source_drawing_path = resolve_target(&source_sheet_path, &source_drawing_target)?;
        let source_drawing_xml = source.entry_xml(&source_drawing_path)?;
        let source_anchor = image_anchor(&source_drawing_xml)
            .ok_or_else(|| Error::Xlsx("compiled drawing has no image anchor".to_owned()))?;
        let source_image_id = drawing_image_relationship_id(&source_anchor)
            .ok_or_else(|| Error::Xlsx("compiled image anchor has no relationship id".to_owned()))?;
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
                .ok_or_else(|| Error::Xlsx("template drawing relationship is missing".to_owned()))?;
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
            self.entry_mut(&target_drawing_path)?.bytes = insert_before_close_tag(
                &drawing_xml,
                "</xdr:wsDr>",
                &imported_anchor,
            )?
            .into_bytes();
            let relationship = format!(
                concat!(
                    "<Relationship Id=\"{new_image_id}\" ",
                    "Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/image\" ",
                    "Target=\"../media/image{image_number}.{extension}\"/>"
                )
            );
            target_drawing_rels = insert_before_close_tag(
                &target_drawing_rels,
                "</Relationships>",
                &relationship,
            )?;
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
                concat!(
                    "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>",
                    "<Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\">",
                    "<Relationship Id=\"rId1\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/image\" ",
                    "Target=\"../media/image{image_number}.{extension}\"/>",
                    "</Relationships>"
                )
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
            .ok_or_else(|| Error::Xlsx("compiled comment VML relationship is missing".to_owned()))?;
        let source_comments_path = resolve_target(&source_sheet_path, &source_comments_target)?;
        let source_vml_path = resolve_target(&source_sheet_path, &source_vml_target)?;
        let source_comments = source.entry_xml(&source_comments_path)?;
        let source_vml = source.entry_xml(&source_vml_path)?;
        let reference = a1_reference(row_index, column_index);
        let source_comment = xml_element_by_attribute(
            &source_comments,
            "comment",
            "ref",
            &reference,
        )?
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
            let vml_target = relationship_target_by_type(&target_rels, "/vmlDrawing")
                .ok_or_else(|| Error::Xlsx("template comments have no VML relationship".to_owned()))?;
            let vml_path = resolve_target(&target_sheet_path, &vml_target)?;
            let comments_xml = self.entry_xml(&comments_path)?;
            let source_author_id = attribute_value(&source_comment, "authorId")
                .and_then(|value| value.parse::<usize>().ok())
                .ok_or_else(|| Error::Xlsx("compiled comment authorId is invalid".to_owned()))?;
            let source_author = xml_element_inner_by_index(&source_comments, "author", source_author_id)
                .ok_or_else(|| Error::Xlsx("compiled comment author is missing".to_owned()))?;
            let target_authors = xml_element_inners(&comments_xml, "author");
            let (comments_xml, target_author_id) = if let Some(index) =
                target_authors.iter().position(|author| *author == source_author)
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
            let source_comment = replace_xml_attribute(
                &source_comment,
                "authorId",
                &target_author_id.to_string(),
            )?;
            self.entry_mut(&comments_path)?.bytes = insert_before_close_tag(
                &comments_xml,
                "</commentList>",
                &source_comment,
            )?
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
        target_rels = insert_before_close_tag(
            &target_rels,
            "</Relationships>",
            &comments_relationship,
        )?;
        let vml_id = next_relationship_id(&target_rels);
        let vml_relationship = format!(
            "<Relationship Id=\"{vml_id}\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/vmlDrawing\" Target=\"../drawings/vmlDrawing{vml_number}.vml\"/>"
        );
        target_rels = insert_before_close_tag(
            &target_rels,
            "</Relationships>",
            &vml_relationship,
        )?;
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
            .ok_or_else(|| Error::Xlsx("compiled chart drawing relationship is missing".to_owned()))?;
        let source_drawing_path = resolve_target(&source_sheet_path, &source_drawing_target)?;
        let source_drawing_xml = source.entry_xml(&source_drawing_path)?;
        let source_anchor = chart_anchor(&source_drawing_xml)
            .ok_or_else(|| Error::Xlsx("compiled drawing has no chart anchor".to_owned()))?;
        let source_anchor_chart_id = drawing_chart_relationship_id(&source_anchor)
            .ok_or_else(|| Error::Xlsx("compiled chart anchor has no relationship id".to_owned()))?;
        let source_drawing_rels_path = relationship_part_name(&source_drawing_path);
        let source_drawing_rels = source.entry_xml(&source_drawing_rels_path)?;
        let source_chart_target = relationship_target(&source_drawing_rels, &source_anchor_chart_id)
            .ok_or_else(|| Error::Xlsx("compiled drawing chart relationship is missing".to_owned()))?;
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
                .ok_or_else(|| Error::Xlsx("template drawing relationship is missing".to_owned()))?;
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
            drawing_entry.bytes = insert_before_close_tag(
                &drawing_xml,
                "</xdr:wsDr>",
                &imported_anchor,
            )?
            .into_bytes();
            let relationship = format!(
                "<Relationship Id=\"{new_chart_id}\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/chart\" Target=\"../charts/chart{chart_number}.xml\"/>"
            );
            target_drawing_rels = insert_before_close_tag(
                &target_drawing_rels,
                "</Relationships>",
                &relationship,
            )?;
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
                concat!(
                    "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>",
                    "<Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\">",
                    "<Relationship Id=\"rId1\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/chart\" ",
                    "Target=\"../charts/chart{chart_number}.xml\"/>",
                    "</Relationships>"
                )
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
            + xml[start..]
                .find("/>")
                .ok_or_else(|| Error::Xlsx("template hyperlinks element is malformed".to_owned()))?
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
        if xml.as_bytes().get(name_end).is_some_and(|byte| !byte.is_ascii_whitespace()) {
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
        if xml.as_bytes().get(name_end).is_some_and(|byte| !byte.is_ascii_whitespace()) {
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
        if xml.as_bytes().get(name_end).is_some_and(|byte| !byte.is_ascii_whitespace() && *byte != b'>') {
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
    for tag in ["xdr:twoCellAnchor", "xdr:oneCellAnchor", "xdr:absoluteAnchor"] {
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
    for tag in ["xdr:twoCellAnchor", "xdr:oneCellAnchor", "xdr:absoluteAnchor"] {
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
    let override_tag = format!(
        "<Override PartName=\"{part_name}\" ContentType=\"{content_type}\"/>"
    );
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
    let default_tag = format!(
        "<Default Extension=\"{extension}\" ContentType=\"{content_type}\"/>"
    );
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
        EMPTY_WORKSHEET_XML, OoxmlZipEntry, blank_worksheet_with_inherited_format,
        extract_xml_element, insert_before_close_tag, next_relationship_id, next_sheet_id,
        next_worksheet_part_name,
    };
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
}
