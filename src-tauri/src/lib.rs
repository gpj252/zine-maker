mod imposition;
mod pdf_export;
mod project;

use project::{compute_imposition, ImpositionResult, Project};

#[tauri::command]
fn get_imposition(project: Project) -> ImpositionResult {
    compute_imposition(&project)
}

#[tauri::command]
fn export_project_pdf(project: Project, output_path: String) -> Result<(), String> {
    let result = compute_imposition(&project);
    // Pad the page list the same way compute_imposition padded the sheet count, so the sheets'
    // page-number references (which may point past the end of `project.pages` by
    // `auto_blanks_added`) resolve to real — blank — entries instead of panicking.
    let mut pages = project.pages.clone();
    for _ in 0..result.auto_blanks_added {
        pages.push(project::PageEntry { id: uuid::Uuid::new_v4().to_string(), image_path: None });
    }
    pdf_export::export_pdf(
        &pages,
        &result.sheets,
        project.paper.width_mm,
        project.paper.height_mm,
        (
            project.margins.top_mm,
            project.margins.right_mm,
            project.margins.bottom_mm,
            project.margins.left_mm,
        ),
        &output_path,
    )
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![get_imposition, export_project_pdf])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
