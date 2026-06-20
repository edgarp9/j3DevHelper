use super::*;

const WORKSPACE_DIALOG_CLASS_NAME: &str = "j3DevHelper.WorkspaceDialog";
const TEXT_INPUT_DIALOG_CLASS_NAME: &str = "j3DevHelper.TextInputDialog";
const COMMAND_BUTTON_DIALOG_CLASS_NAME: &str = "j3DevHelper.CommandButtonDialog";
const FONT_DIALOG_CLASS_NAME: &str = "j3DevHelper.FontDialog";
const LANGUAGE_CONFIG_DIALOG_CLASS_NAME: &str = "j3DevHelper.LanguageConfigDialog";
const ABOUT_DIALOG_CLASS_NAME: &str = "j3DevHelper.AboutDialog";
const WORKSPACE_DIALOG_WIDTH: i32 = 460;
const WORKSPACE_DIALOG_HEIGHT: i32 = 236;
const TEXT_INPUT_DIALOG_WIDTH: i32 = 390;
const TEXT_INPUT_DIALOG_HEIGHT: i32 = 154;
const COMMAND_BUTTON_DIALOG_WIDTH: i32 = 570;
const COMMAND_BUTTON_DIALOG_HEIGHT: i32 = 360;
const FONT_DIALOG_WIDTH: i32 = 500;
const FONT_DIALOG_HEIGHT: i32 = 292;
const LANGUAGE_CONFIG_DIALOG_WIDTH: i32 = 460;
const LANGUAGE_CONFIG_DIALOG_HEIGHT: i32 = 330;
const ABOUT_DIALOG_WIDTH: i32 = 320;
const ABOUT_DIALOG_HEIGHT: i32 = 160;
const WM_WORKSPACE_DIALOG_PATH_CHECKED: u32 = WM_APP + 1;
const WM_WORKSPACE_DIALOG_LANGUAGE_INFERRED: u32 = WM_APP + 2;
const CONTROL_NAME_EDIT_ID: i32 = 2001;
const CONTROL_PATH_EDIT_ID: i32 = 2002;
const CONTROL_BROWSE_BUTTON_ID: i32 = 2003;
const CONTROL_LANGUAGE_COMBO_ID: i32 = 2004;
const CONTROL_TEXT_INPUT_EDIT_ID: i32 = 3001;
const CONTROL_COMMAND_BUTTON_NAME_EDIT_ID: i32 = 5001;
const CONTROL_EXECUTABLE_PATH_EDIT_ID: i32 = 5002;
const CONTROL_EXECUTABLE_BROWSE_BUTTON_ID: i32 = 5003;
const CONTROL_ARGUMENTS_EDIT_ID: i32 = 5004;
const CONTROL_EXECUTION_TYPE_SHELL_RADIO_ID: i32 = 5005;
const CONTROL_EXECUTION_TYPE_EXTERNAL_RADIO_ID: i32 = 5006;
const CONTROL_TOKEN_PATH_BUTTON_ID: i32 = 5010;
const CONTROL_TOKEN_NAME_BUTTON_ID: i32 = 5011;
const CONTROL_TOKEN_SELECT_FILE_BUTTON_ID: i32 = 5012;
const CONTROL_TOKEN_SELECT_DIR_BUTTON_ID: i32 = 5013;
const CONTROL_TOKEN_INPUT_TEXT_BUTTON_ID: i32 = 5014;
const CONTROL_TOKEN_LANGUAGE_BUTTON_ID: i32 = 5015;
const CONTROL_APPLY_BUTTON_ID: i32 = 5020;
const CONTROL_FONT_FAMILY_COMBO_ID: i32 = 7001;
const CONTROL_FONT_SIZE_COMBO_ID: i32 = 7002;
const CONTROL_FONT_DEFAULT_BUTTON_ID: i32 = 7003;
const CONTROL_FONT_APPLY_BUTTON_ID: i32 = 7004;
const CONTROL_LANGUAGE_CONFIG_EDIT_ID: i32 = 8001;
const CONTROL_LANGUAGE_CONFIG_DEFAULT_BUTTON_ID: i32 = 8002;
const CONTROL_ABOUT_LINK_ID: i32 = 9001;
static NEXT_WORKSPACE_DIALOG_REQUEST_ID: std::sync::atomic::AtomicU32 =
    std::sync::atomic::AtomicU32::new(1);

fn font_preview_text(language: UiLanguage) -> &'static str {
    tr(language, "미리보기 123 가나다", "Preview 123 ABC")
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum WorkspaceDialogMode {
    Add,
    Edit,
}

impl WorkspaceDialogMode {
    fn title(self, language: UiLanguage) -> &'static str {
        match language {
            UiLanguage::Korean => match self {
                Self::Add => "워크스페이스 추가",
                Self::Edit => "워크스페이스 편집",
            },
            UiLanguage::English => match self {
                Self::Add => "Add Workspace",
                Self::Edit => "Edit Workspace",
            },
        }
    }
}

struct WorkspaceDialogContext {
    mode: WorkspaceDialogMode,
    initial_workspace: Option<Workspace>,
    reserved_paths: Vec<String>,
    language_options: Vec<String>,
    language: UiLanguage,
    controls: WorkspaceDialogControls,
    result: Option<Workspace>,
    previous_folder_default_name: Option<String>,
    pending_path_validation: Option<PendingWorkspacePathValidation>,
    pending_language_inference: Option<u32>,
}

impl WorkspaceDialogContext {
    fn new(
        mode: WorkspaceDialogMode,
        initial_workspace: Option<Workspace>,
        reserved_paths: Vec<String>,
        language_options: Vec<String>,
        language: UiLanguage,
    ) -> Self {
        Self {
            mode,
            initial_workspace,
            reserved_paths,
            language_options,
            language,
            controls: WorkspaceDialogControls::empty(),
            result: None,
            previous_folder_default_name: None,
            pending_path_validation: None,
            pending_language_inference: None,
        }
    }
}

struct PendingWorkspacePathValidation {
    request_id: u32,
    path: String,
    name: String,
    language: Option<String>,
}

struct WorkspaceDialogControls {
    name_label: HWND,
    name_edit: HWND,
    path_label: HWND,
    path_edit: HWND,
    browse_button: HWND,
    language_label: HWND,
    language_combo: HWND,
    ok_button: HWND,
    cancel_button: HWND,
}

impl WorkspaceDialogControls {
    fn empty() -> Self {
        Self {
            name_label: null_mut(),
            name_edit: null_mut(),
            path_label: null_mut(),
            path_edit: null_mut(),
            browse_button: null_mut(),
            language_label: null_mut(),
            language_combo: null_mut(),
            ok_button: null_mut(),
            cancel_button: null_mut(),
        }
    }

    fn handles(&self) -> [HWND; 9] {
        [
            self.name_label,
            self.name_edit,
            self.path_label,
            self.path_edit,
            self.browse_button,
            self.language_label,
            self.language_combo,
            self.ok_button,
            self.cancel_button,
        ]
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum CommandButtonDialogMode {
    Add,
    Edit,
}

impl CommandButtonDialogMode {
    fn title(self, language: UiLanguage) -> &'static str {
        match language {
            UiLanguage::Korean => match self {
                Self::Add => "명령 추가",
                Self::Edit => "명령 편집",
            },
            UiLanguage::English => match self {
                Self::Add => "Add Command",
                Self::Edit => "Edit Command",
            },
        }
    }
}

struct CommandButtonDialogContext {
    mode: CommandButtonDialogMode,
    initial_button: Option<CommandButton>,
    tab_index: usize,
    button_index: Option<usize>,
    owner: HWND,
    app_context: *mut WindowContext,
    language: UiLanguage,
    controls: CommandButtonDialogControls,
}

impl CommandButtonDialogContext {
    fn new(
        mode: CommandButtonDialogMode,
        initial_button: Option<CommandButton>,
        tab_index: usize,
        button_index: Option<usize>,
        owner: HWND,
        app_context: *mut WindowContext,
        language: UiLanguage,
    ) -> Self {
        Self {
            mode,
            initial_button,
            tab_index,
            button_index,
            owner,
            app_context,
            language,
            controls: CommandButtonDialogControls::empty(),
        }
    }
}

struct CommandButtonDialogControls {
    button_name_label: HWND,
    button_name_edit: HWND,
    executable_path_label: HWND,
    executable_path_edit: HWND,
    executable_browse_button: HWND,
    arguments_label: HWND,
    arguments_edit: HWND,
    token_path_button: HWND,
    token_name_button: HWND,
    token_select_file_button: HWND,
    token_select_dir_button: HWND,
    token_input_text_button: HWND,
    token_language_button: HWND,
    execution_type_label: HWND,
    shell_api_radio: HWND,
    external_terminal_radio: HWND,
    ok_button: HWND,
    cancel_button: HWND,
    apply_button: HWND,
}

impl CommandButtonDialogControls {
    fn empty() -> Self {
        Self {
            button_name_label: null_mut(),
            button_name_edit: null_mut(),
            executable_path_label: null_mut(),
            executable_path_edit: null_mut(),
            executable_browse_button: null_mut(),
            arguments_label: null_mut(),
            arguments_edit: null_mut(),
            token_path_button: null_mut(),
            token_name_button: null_mut(),
            token_select_file_button: null_mut(),
            token_select_dir_button: null_mut(),
            token_input_text_button: null_mut(),
            token_language_button: null_mut(),
            execution_type_label: null_mut(),
            shell_api_radio: null_mut(),
            external_terminal_radio: null_mut(),
            ok_button: null_mut(),
            cancel_button: null_mut(),
            apply_button: null_mut(),
        }
    }

    fn handles(&self) -> [HWND; 19] {
        [
            self.button_name_label,
            self.button_name_edit,
            self.executable_path_label,
            self.executable_path_edit,
            self.executable_browse_button,
            self.arguments_label,
            self.arguments_edit,
            self.token_path_button,
            self.token_name_button,
            self.token_select_file_button,
            self.token_select_dir_button,
            self.token_input_text_button,
            self.token_language_button,
            self.execution_type_label,
            self.shell_api_radio,
            self.external_terminal_radio,
            self.ok_button,
            self.cancel_button,
            self.apply_button,
        ]
    }
}

struct TextInputDialogContext {
    title: String,
    prompt: String,
    initial_value: String,
    allow_empty: bool,
    language: UiLanguage,
    controls: TextInputDialogControls,
    result: Option<String>,
}

impl TextInputDialogContext {
    fn new_with_options(
        title: &str,
        prompt: &str,
        initial_value: &str,
        allow_empty: bool,
        language: UiLanguage,
    ) -> Self {
        Self {
            title: title.to_owned(),
            prompt: prompt.to_owned(),
            initial_value: initial_value.to_owned(),
            allow_empty,
            language,
            controls: TextInputDialogControls::empty(),
            result: None,
        }
    }
}

struct TextInputDialogControls {
    prompt_label: HWND,
    value_edit: HWND,
    ok_button: HWND,
    cancel_button: HWND,
}

impl TextInputDialogControls {
    fn empty() -> Self {
        Self {
            prompt_label: null_mut(),
            value_edit: null_mut(),
            ok_button: null_mut(),
            cancel_button: null_mut(),
        }
    }

    fn handles(&self) -> [HWND; 4] {
        [
            self.prompt_label,
            self.value_edit,
            self.ok_button,
            self.cancel_button,
        ]
    }
}

pub(super) struct TextInputDialogSpec<'a> {
    title: &'a str,
    prompt: &'a str,
    initial_value: &'a str,
}

impl<'a> TextInputDialogSpec<'a> {
    pub(super) fn new(title: &'a str, prompt: &'a str, initial_value: &'a str) -> Self {
        Self {
            title,
            prompt,
            initial_value,
        }
    }
}

struct FontDialogContext {
    fonts: Vec<String>,
    initial_view: ViewSettings,
    dpi: u32,
    language: UiLanguage,
    controls: FontDialogControls,
    result: Option<ViewSettings>,
    preview_font: UiFont,
}

impl FontDialogContext {
    fn new(fonts: Vec<String>, initial_view: ViewSettings, dpi: u32, language: UiLanguage) -> Self {
        let preview_font = UiFont::from_view(&initial_view, dpi);
        Self {
            fonts,
            initial_view,
            dpi,
            language,
            controls: FontDialogControls::empty(),
            result: None,
            preview_font,
        }
    }
}

struct FontDialogControls {
    font_label: HWND,
    font_combo: HWND,
    size_label: HWND,
    size_combo: HWND,
    preview_label: HWND,
    default_button: HWND,
    apply_button: HWND,
    cancel_button: HWND,
}

impl FontDialogControls {
    fn empty() -> Self {
        Self {
            font_label: null_mut(),
            font_combo: null_mut(),
            size_label: null_mut(),
            size_combo: null_mut(),
            preview_label: null_mut(),
            default_button: null_mut(),
            apply_button: null_mut(),
            cancel_button: null_mut(),
        }
    }

    fn handles(&self) -> [HWND; 8] {
        [
            self.font_label,
            self.font_combo,
            self.size_label,
            self.size_combo,
            self.preview_label,
            self.default_button,
            self.apply_button,
            self.cancel_button,
        ]
    }
}

struct LanguageConfigDialogContext {
    initial_languages: Vec<String>,
    language: UiLanguage,
    controls: LanguageConfigDialogControls,
    result: Option<Vec<String>>,
}

impl LanguageConfigDialogContext {
    fn new(initial_languages: Vec<String>, language: UiLanguage) -> Self {
        Self {
            initial_languages,
            language,
            controls: LanguageConfigDialogControls::empty(),
            result: None,
        }
    }
}

struct LanguageConfigDialogControls {
    languages_label: HWND,
    languages_edit: HWND,
    default_button: HWND,
    ok_button: HWND,
    cancel_button: HWND,
}

impl LanguageConfigDialogControls {
    fn empty() -> Self {
        Self {
            languages_label: null_mut(),
            languages_edit: null_mut(),
            default_button: null_mut(),
            ok_button: null_mut(),
            cancel_button: null_mut(),
        }
    }

    fn handles(&self) -> [HWND; 5] {
        [
            self.languages_label,
            self.languages_edit,
            self.default_button,
            self.ok_button,
            self.cancel_button,
        ]
    }
}

struct AboutDialogContext {
    language: UiLanguage,
    controls: AboutDialogControls,
}

impl AboutDialogContext {
    fn new(language: UiLanguage) -> Self {
        Self {
            language,
            controls: AboutDialogControls::empty(),
        }
    }
}

struct AboutDialogControls {
    message_label: HWND,
    repository_link: HWND,
    ok_button: HWND,
}

impl AboutDialogControls {
    fn empty() -> Self {
        Self {
            message_label: null_mut(),
            repository_link: null_mut(),
            ok_button: null_mut(),
        }
    }

    fn handles(&self) -> [HWND; 3] {
        [self.message_label, self.repository_link, self.ok_button]
    }
}

struct ModalDialogSpec<'a> {
    class_name: &'a str,
    caption: &'a str,
    width: i32,
    height: i32,
    font: HFONT,
    font_size: u16,
    create_error_context: &'static str,
}

impl<'a> ModalDialogSpec<'a> {
    fn new(
        class_name: &'a str,
        caption: &'a str,
        width: i32,
        height: i32,
        font: HFONT,
        font_size: u16,
        create_error_context: &'static str,
    ) -> Self {
        Self {
            class_name,
            caption,
            width,
            height,
            font,
            font_size: normalize_ui_font_size(font_size),
            create_error_context,
        }
    }
}

struct ModalDialogWindow {
    owner: HWND,
    hwnd: HWND,
    owner_disabled_by_us: bool,
}

impl ModalDialogWindow {
    fn new(owner: HWND, hwnd: HWND) -> Self {
        Self {
            owner,
            hwnd,
            owner_disabled_by_us: false,
        }
    }

    fn hwnd(&self) -> HWND {
        self.hwnd
    }

    fn show_modal(&mut self, message_loop_error_context: &'static str) -> AppResult<()> {
        self.disable_owner();

        // SAFETY: owner and hwnd are windows on this UI thread. Disabling the owner enforces
        // modality, and the dialog stays alive until the modal loop exits or Drop cleans it up.
        unsafe {
            ShowWindow(self.hwnd, SW_SHOW);
            UpdateWindow(self.hwnd);
        }

        let result = modal_dialog_message_loop(self.hwnd, message_loop_error_context);
        let owner_was_disabled = self.enable_owner();
        self.destroy_if_open();
        if owner_was_disabled {
            self.restore_owner_foreground();
        }
        result
    }

    fn disable_owner(&mut self) {
        // SAFETY: owner is the parent window supplied to CreateWindowExW. IsWindowEnabled only
        // reads the current enabled state.
        let owner_was_enabled = unsafe { IsWindowEnabled(self.owner) != 0 };
        if owner_was_enabled {
            // SAFETY: owner is the parent window on this UI thread.
            unsafe {
                EnableWindow(self.owner, 0);
            }
            self.owner_disabled_by_us = true;
        }
    }

    fn enable_owner(&mut self) -> bool {
        if !self.owner_disabled_by_us {
            return false;
        }

        // SAFETY: owner was disabled by this guard and should be restored when the modal scope
        // ends, including early returns.
        unsafe {
            EnableWindow(self.owner, 1);
        }
        self.owner_disabled_by_us = false;
        true
    }

    fn restore_owner_foreground(&self) {
        // SAFETY: owner is the parent window supplied to CreateWindowExW. If the modal window was
        // destroyed while the owner was disabled, Windows may have activated the previous app; make
        // the owner active again after it has been re-enabled.
        unsafe {
            if IsWindow(self.owner) != 0 {
                SetActiveWindow(self.owner);
                SetForegroundWindow(self.owner);
            }
        }
    }

    fn destroy_if_open(&mut self) {
        // SAFETY: hwnd may already have been destroyed by the dialog procedure. IsWindow is used
        // only to avoid calling DestroyWindow on a stale handle.
        let is_open = unsafe { IsWindow(self.hwnd) };
        if is_open == 0 {
            return;
        }

        // SAFETY: hwnd is a dialog window created on this thread. DestroyWindow sends
        // WM_NCDESTROY synchronously for this window, where the non-owning context pointer is
        // cleared.
        unsafe {
            clear_dialog_font_size(self.hwnd);
            DestroyWindow(self.hwnd);
        }
    }
}

impl Drop for ModalDialogWindow {
    fn drop(&mut self) {
        let owner_was_disabled = self.enable_owner();
        self.destroy_if_open();
        if owner_was_disabled {
            self.restore_owner_foreground();
        }
    }
}

fn create_modal_dialog_window<T>(
    owner: HWND,
    instance: HINSTANCE,
    spec: ModalDialogSpec<'_>,
    context: &mut T,
) -> AppResult<ModalDialogWindow> {
    let class_name = wide_null(spec.class_name);
    let caption = wide_null(spec.caption);
    let dpi = window_dpi(owner);
    let layout_font_size = dialog_layout_font_size(spec.font, spec.font_size, dpi);
    let (width, height) =
        scaled_dialog_size_for_dpi(spec.width, spec.height, layout_font_size, dpi);
    let (x, y) = centered_window_position(owner, width, height);

    // SAFETY: The dialog class is registered, strings are null-terminated, and every caller keeps
    // the boxed context alive until the returned ModalDialogWindow is dropped.
    let hwnd = unsafe {
        CreateWindowExW(
            WS_EX_DLGMODALFRAME | WS_EX_CONTROLPARENT,
            class_name.as_ptr(),
            caption.as_ptr(),
            WS_POPUP | WS_CAPTION | WS_SYSMENU,
            x,
            y,
            width,
            height,
            owner,
            null_mut(),
            instance,
            context as *mut T as *const c_void,
        )
    };

    if hwnd.is_null() {
        Err(last_error(spec.create_error_context))
    } else if let Err(error) = set_dialog_font_size(hwnd, layout_font_size) {
        // SAFETY: hwnd is a newly created top-level dialog and is not yet visible.
        unsafe {
            DestroyWindow(hwnd);
        }
        Err(error)
    } else {
        Ok(ModalDialogWindow::new(owner, hwnd))
    }
}

pub(super) fn show_about_dialog(
    owner: HWND,
    instance: HINSTANCE,
    app_title: &str,
    version: &str,
    font: HFONT,
    font_size: u16,
    language: UiLanguage,
) -> AppResult<()> {
    register_about_dialog_class(instance)?;

    let mut context = Box::new(AboutDialogContext::new(language));
    let mut dialog = create_modal_dialog_window(
        owner,
        instance,
        ModalDialogSpec::new(
            ABOUT_DIALOG_CLASS_NAME,
            tr(language, "정보", "About"),
            ABOUT_DIALOG_WIDTH,
            ABOUT_DIALOG_HEIGHT,
            font,
            font_size,
            "CreateWindowExW about dialog",
        ),
        context.as_mut(),
    )?;

    let message = format!("{app_title}  v{version}");
    create_about_dialog_controls(dialog.hwnd(), instance, context.as_mut(), &message)?;

    apply_font_to_handles(&context.controls.handles(), font);
    dialog.show_modal("GetMessageW about dialog")
}

#[allow(clippy::too_many_arguments)]
pub(super) fn show_workspace_dialog(
    owner: HWND,
    instance: HINSTANCE,
    mode: WorkspaceDialogMode,
    initial_workspace: Option<Workspace>,
    reserved_paths: Vec<String>,
    language_options: Vec<String>,
    font: HFONT,
    font_size: u16,
    language: UiLanguage,
) -> AppResult<Option<Workspace>> {
    register_workspace_dialog_class(instance)?;

    let mut context = Box::new(WorkspaceDialogContext::new(
        mode,
        initial_workspace,
        reserved_paths,
        language_options,
        language,
    ));
    let mut dialog = create_modal_dialog_window(
        owner,
        instance,
        ModalDialogSpec::new(
            WORKSPACE_DIALOG_CLASS_NAME,
            mode.title(language),
            WORKSPACE_DIALOG_WIDTH,
            WORKSPACE_DIALOG_HEIGHT,
            font,
            font_size,
            "CreateWindowExW workspace dialog",
        ),
        context.as_mut(),
    )?;

    create_workspace_dialog_controls(dialog.hwnd(), instance, context.as_mut())?;
    initialize_workspace_dialog_values(context.as_mut());
    apply_font_to_handles(&context.controls.handles(), font);

    dialog.show_modal("GetMessageW workspace dialog")?;
    Ok(context.result.take())
}

// Dialog creation carries Win32 parent/instance/mode/state handles that are clearer at call sites
// than behind a broad builder for this small modal surface.
#[allow(clippy::too_many_arguments)]
pub(super) fn show_command_button_dialog(
    owner: HWND,
    instance: HINSTANCE,
    mode: CommandButtonDialogMode,
    initial_button: Option<CommandButton>,
    tab_index: usize,
    button_index: Option<usize>,
    app_context: *mut WindowContext,
    font: HFONT,
    font_size: u16,
    language: UiLanguage,
) -> AppResult<()> {
    register_command_button_dialog_class(instance)?;

    let mut context = Box::new(CommandButtonDialogContext::new(
        mode,
        initial_button,
        tab_index,
        button_index,
        owner,
        app_context,
        language,
    ));
    let mut dialog = create_modal_dialog_window(
        owner,
        instance,
        ModalDialogSpec::new(
            COMMAND_BUTTON_DIALOG_CLASS_NAME,
            mode.title(language),
            COMMAND_BUTTON_DIALOG_WIDTH,
            COMMAND_BUTTON_DIALOG_HEIGHT,
            font,
            font_size,
            "CreateWindowExW command button dialog",
        ),
        context.as_mut(),
    )?;

    create_command_button_dialog_controls(dialog.hwnd(), instance, context.as_mut())?;
    initialize_command_button_dialog_values(dialog.hwnd(), context.as_mut());
    apply_font_to_handles(&context.controls.handles(), font);
    dialog.show_modal("GetMessageW command button dialog")
}

pub(super) fn show_font_dialog(
    owner: HWND,
    instance: HINSTANCE,
    initial_view: &ViewSettings,
    fonts: Vec<String>,
    font: HFONT,
    dpi: u32,
    language: UiLanguage,
) -> AppResult<Option<ViewSettings>> {
    register_font_dialog_class(instance)?;

    let mut context = Box::new(FontDialogContext::new(
        fonts,
        initial_view.clone(),
        dpi,
        language,
    ));
    let mut dialog = create_modal_dialog_window(
        owner,
        instance,
        ModalDialogSpec::new(
            FONT_DIALOG_CLASS_NAME,
            tr(language, "글꼴", "Font"),
            FONT_DIALOG_WIDTH,
            FONT_DIALOG_HEIGHT,
            font,
            initial_view.font_size,
            "CreateWindowExW font dialog",
        ),
        context.as_mut(),
    )?;

    create_font_dialog_controls(dialog.hwnd(), instance, context.as_mut())?;
    initialize_font_dialog_values(dialog.hwnd(), context.as_mut());
    apply_font_to_handles(&context.controls.handles(), font);
    font_dialog_update_preview(dialog.hwnd(), context.as_mut());

    dialog.show_modal("GetMessageW font dialog")?;
    Ok(context.result.take())
}

pub(super) fn show_language_config_dialog(
    owner: HWND,
    instance: HINSTANCE,
    initial_languages: Vec<String>,
    font: HFONT,
    font_size: u16,
    language: UiLanguage,
) -> AppResult<Option<Vec<String>>> {
    register_language_config_dialog_class(instance)?;

    let mut context = Box::new(LanguageConfigDialogContext::new(
        initial_languages,
        language,
    ));
    let mut dialog = create_modal_dialog_window(
        owner,
        instance,
        ModalDialogSpec::new(
            LANGUAGE_CONFIG_DIALOG_CLASS_NAME,
            tr(language, "워크스페이스 언어", "Workspace Languages"),
            LANGUAGE_CONFIG_DIALOG_WIDTH,
            LANGUAGE_CONFIG_DIALOG_HEIGHT,
            font,
            font_size,
            "CreateWindowExW language config dialog",
        ),
        context.as_mut(),
    )?;

    create_language_config_dialog_controls(dialog.hwnd(), instance, context.as_mut())?;
    initialize_language_config_dialog_values(context.as_mut());
    apply_font_to_handles(&context.controls.handles(), font);

    dialog.show_modal("GetMessageW language config dialog")?;
    Ok(context.result.take())
}

pub(super) fn show_text_input_dialog(
    owner: HWND,
    instance: HINSTANCE,
    spec: TextInputDialogSpec<'_>,
    font: HFONT,
    font_size: u16,
    language: UiLanguage,
) -> AppResult<Option<String>> {
    show_text_input_dialog_with_options(owner, instance, spec, false, font, font_size, language)
}

pub(super) fn show_argument_text_input_dialog(
    owner: HWND,
    instance: HINSTANCE,
    font: HFONT,
    font_size: u16,
    language: UiLanguage,
) -> AppResult<Option<String>> {
    show_text_input_dialog_with_options(
        owner,
        instance,
        TextInputDialogSpec::new(
            tr(language, "텍스트 입력", "Text Input"),
            tr(language, "텍스트", "Text"),
            "",
        ),
        true,
        font,
        font_size,
        language,
    )
}

#[allow(clippy::too_many_arguments)]
fn show_text_input_dialog_with_options(
    owner: HWND,
    instance: HINSTANCE,
    spec: TextInputDialogSpec<'_>,
    allow_empty: bool,
    font: HFONT,
    font_size: u16,
    language: UiLanguage,
) -> AppResult<Option<String>> {
    register_text_input_dialog_class(instance)?;

    let mut context = Box::new(TextInputDialogContext::new_with_options(
        spec.title,
        spec.prompt,
        spec.initial_value,
        allow_empty,
        language,
    ));
    let mut dialog = create_modal_dialog_window(
        owner,
        instance,
        ModalDialogSpec::new(
            TEXT_INPUT_DIALOG_CLASS_NAME,
            spec.title,
            TEXT_INPUT_DIALOG_WIDTH,
            TEXT_INPUT_DIALOG_HEIGHT,
            font,
            font_size,
            "CreateWindowExW text input dialog",
        ),
        context.as_mut(),
    )?;

    create_text_input_dialog_controls(dialog.hwnd(), instance, context.as_mut())?;
    initialize_text_input_dialog_values(context.as_mut());
    apply_font_to_handles(&context.controls.handles(), font);

    dialog.show_modal("GetMessageW text input dialog")?;
    Ok(context.result.take())
}

fn register_workspace_dialog_class(instance: HINSTANCE) -> AppResult<()> {
    let class_name = wide_null(WORKSPACE_DIALOG_CLASS_NAME);
    // SAFETY: Loading the predefined IDC_ARROW cursor with a null instance is the documented
    // Win32 pattern.
    let cursor = unsafe { LoadCursorW(null_mut(), IDC_ARROW) };
    let window_class = WNDCLASSEXW {
        cbSize: size_of::<WNDCLASSEXW>() as u32,
        style: CS_HREDRAW | CS_VREDRAW,
        lpfnWndProc: Some(workspace_dialog_proc),
        cbClsExtra: 0,
        cbWndExtra: 0,
        hInstance: instance,
        hIcon: null_mut(),
        hCursor: cursor,
        hbrBackground: system_color_brush(COLOR_WINDOW),
        lpszMenuName: null(),
        lpszClassName: class_name.as_ptr(),
        hIconSm: null_mut(),
    };

    // SAFETY: window_class points to a fully initialized WNDCLASSEXW value.
    let atom = unsafe { RegisterClassExW(&window_class) };
    if atom == 0 {
        let code = unsafe { GetLastError() };
        if code != ERROR_CLASS_ALREADY_EXISTS {
            return Err(AppError::windows_api(
                "RegisterClassExW workspace dialog",
                code,
            ));
        }
    }

    Ok(())
}

fn register_text_input_dialog_class(instance: HINSTANCE) -> AppResult<()> {
    register_dialog_class(
        instance,
        TEXT_INPUT_DIALOG_CLASS_NAME,
        text_input_dialog_proc,
        "RegisterClassExW text input dialog",
    )
}

fn register_command_button_dialog_class(instance: HINSTANCE) -> AppResult<()> {
    register_dialog_class(
        instance,
        COMMAND_BUTTON_DIALOG_CLASS_NAME,
        command_button_dialog_proc,
        "RegisterClassExW command button dialog",
    )
}

fn register_font_dialog_class(instance: HINSTANCE) -> AppResult<()> {
    register_dialog_class(
        instance,
        FONT_DIALOG_CLASS_NAME,
        font_dialog_proc,
        "RegisterClassExW font dialog",
    )
}

fn register_language_config_dialog_class(instance: HINSTANCE) -> AppResult<()> {
    register_dialog_class(
        instance,
        LANGUAGE_CONFIG_DIALOG_CLASS_NAME,
        language_config_dialog_proc,
        "RegisterClassExW language config dialog",
    )
}

fn register_about_dialog_class(instance: HINSTANCE) -> AppResult<()> {
    register_dialog_class(
        instance,
        ABOUT_DIALOG_CLASS_NAME,
        about_dialog_proc,
        "RegisterClassExW about dialog",
    )
}

fn register_dialog_class(
    instance: HINSTANCE,
    class_name: &str,
    window_proc: unsafe extern "system" fn(HWND, u32, WPARAM, LPARAM) -> LRESULT,
    operation: &'static str,
) -> AppResult<()> {
    let class_name = wide_null(class_name);
    // SAFETY: Loading the predefined IDC_ARROW cursor with a null instance is the documented
    // Win32 pattern.
    let cursor = unsafe { LoadCursorW(null_mut(), IDC_ARROW) };
    let window_class = WNDCLASSEXW {
        cbSize: size_of::<WNDCLASSEXW>() as u32,
        style: CS_HREDRAW | CS_VREDRAW,
        lpfnWndProc: Some(window_proc),
        cbClsExtra: 0,
        cbWndExtra: 0,
        hInstance: instance,
        hIcon: null_mut(),
        hCursor: cursor,
        hbrBackground: system_color_brush(COLOR_WINDOW),
        lpszMenuName: null(),
        lpszClassName: class_name.as_ptr(),
        hIconSm: null_mut(),
    };

    // SAFETY: window_class points to a fully initialized WNDCLASSEXW value.
    let atom = unsafe { RegisterClassExW(&window_class) };
    if atom == 0 {
        let code = unsafe { GetLastError() };
        if code != ERROR_CLASS_ALREADY_EXISTS {
            return Err(AppError::windows_api(operation, code));
        }
    }

    Ok(())
}

fn create_workspace_dialog_controls(
    hwnd: HWND,
    instance: HINSTANCE,
    context: &mut WorkspaceDialogContext,
) -> AppResult<()> {
    let static_class = wide_null("STATIC");
    let edit_class = wide_null("EDIT");
    let button_class = wide_null("BUTTON");
    let combo_class = wide_null("COMBOBOX");

    let label_style = WS_CHILD | WS_VISIBLE;
    let edit_style = WS_CHILD | WS_VISIBLE | WS_TABSTOP | WS_BORDER | ES_AUTOHSCROLL as u32;
    let readonly_path_style = edit_style | ES_READONLY as u32;
    let button_style = WS_CHILD | WS_VISIBLE | WS_TABSTOP | BS_PUSHBUTTON as u32;
    let ok_button_style = WS_CHILD | WS_VISIBLE | WS_TABSTOP | WS_GROUP | BS_DEFPUSHBUTTON as u32;
    let combo_style = WS_CHILD
        | WS_VISIBLE
        | WS_TABSTOP
        | WS_BORDER
        | CBS_DROPDOWNLIST as u32
        | CBS_HASSTRINGS as u32;
    let language = context.language;

    context.controls = WorkspaceDialogControls {
        name_label: create_dialog_child(
            hwnd,
            instance,
            static_class.as_ptr(),
            tr(language, "이름", "Name"),
            label_style,
            0,
            RectSpec {
                x: 18,
                y: 20,
                width: 84,
                height: 22,
            },
            "CreateWindowExW workspace name label",
        )?,
        name_edit: create_dialog_child(
            hwnd,
            instance,
            edit_class.as_ptr(),
            "",
            edit_style,
            CONTROL_NAME_EDIT_ID,
            RectSpec {
                x: 108,
                y: 18,
                width: 316,
                height: 24,
            },
            "CreateWindowExW workspace name edit",
        )?,
        path_label: create_dialog_child(
            hwnd,
            instance,
            static_class.as_ptr(),
            tr(language, "폴더", "Folder"),
            label_style,
            0,
            RectSpec {
                x: 18,
                y: 62,
                width: 84,
                height: 22,
            },
            "CreateWindowExW workspace path label",
        )?,
        path_edit: create_dialog_child(
            hwnd,
            instance,
            edit_class.as_ptr(),
            "",
            readonly_path_style,
            CONTROL_PATH_EDIT_ID,
            RectSpec {
                x: 108,
                y: 60,
                width: 236,
                height: 24,
            },
            "CreateWindowExW workspace path edit",
        )?,
        browse_button: create_dialog_child(
            hwnd,
            instance,
            button_class.as_ptr(),
            tr(language, "찾기", "Browse"),
            button_style,
            CONTROL_BROWSE_BUTTON_ID,
            RectSpec {
                x: 354,
                y: 59,
                width: 70,
                height: 26,
            },
            "CreateWindowExW workspace browse button",
        )?,
        language_label: create_dialog_child(
            hwnd,
            instance,
            static_class.as_ptr(),
            tr(language, "언어", "Language"),
            label_style,
            0,
            RectSpec {
                x: 18,
                y: 104,
                width: 84,
                height: 22,
            },
            "CreateWindowExW workspace language label",
        )?,
        language_combo: create_dialog_child(
            hwnd,
            instance,
            combo_class.as_ptr(),
            "",
            combo_style,
            CONTROL_LANGUAGE_COMBO_ID,
            RectSpec {
                x: 108,
                y: 101,
                width: 316,
                height: 180,
            },
            "CreateWindowExW workspace language combo",
        )?,
        ok_button: create_dialog_child(
            hwnd,
            instance,
            button_class.as_ptr(),
            tr(language, "저장", "Save"),
            ok_button_style,
            IDOK,
            RectSpec {
                x: 268,
                y: 156,
                width: 74,
                height: 28,
            },
            "CreateWindowExW workspace ok button",
        )?,
        cancel_button: create_dialog_child(
            hwnd,
            instance,
            button_class.as_ptr(),
            tr(language, "취소", "Cancel"),
            button_style,
            IDCANCEL,
            RectSpec {
                x: 350,
                y: 156,
                width: 74,
                height: 28,
            },
            "CreateWindowExW workspace cancel button",
        )?,
    };

    Ok(())
}

fn create_command_button_dialog_controls(
    hwnd: HWND,
    instance: HINSTANCE,
    context: &mut CommandButtonDialogContext,
) -> AppResult<()> {
    let static_class = wide_null("STATIC");
    let edit_class = wide_null("EDIT");
    let button_class = wide_null("BUTTON");

    let label_style = WS_CHILD | WS_VISIBLE;
    let edit_style = WS_CHILD | WS_VISIBLE | WS_TABSTOP | WS_BORDER | ES_AUTOHSCROLL as u32;
    let button_style = WS_CHILD | WS_VISIBLE | WS_TABSTOP | BS_PUSHBUTTON as u32;
    let ok_button_style = WS_CHILD | WS_VISIBLE | WS_TABSTOP | WS_GROUP | BS_DEFPUSHBUTTON as u32;
    let shell_radio_style =
        WS_CHILD | WS_VISIBLE | WS_TABSTOP | WS_GROUP | BS_AUTORADIOBUTTON as u32;
    let radio_style = WS_CHILD | WS_VISIBLE | WS_TABSTOP | BS_AUTORADIOBUTTON as u32;
    let language = context.language;
    let token_button_width = 156;
    let token_button_height = 28;
    let token_column_gap = 12;
    let token_first_column_x = 18;
    let token_second_column_x = token_first_column_x + token_button_width + token_column_gap;
    let token_third_column_x = token_second_column_x + token_button_width + token_column_gap;
    let token_first_row_y = 140;
    let token_second_row_y = 174;
    let execution_type_y = 222;
    let execution_type_label_width = 150;
    let execution_type_shell_x = 180;
    let execution_type_external_x = 340;

    context.controls = CommandButtonDialogControls {
        button_name_label: create_dialog_child(
            hwnd,
            instance,
            static_class.as_ptr(),
            tr(language, "이름", "Name"),
            label_style,
            0,
            RectSpec {
                x: 18,
                y: 20,
                width: 108,
                height: 22,
            },
            "CreateWindowExW command button name label",
        )?,
        button_name_edit: create_dialog_child(
            hwnd,
            instance,
            edit_class.as_ptr(),
            "",
            edit_style,
            CONTROL_COMMAND_BUTTON_NAME_EDIT_ID,
            RectSpec {
                x: 132,
                y: 18,
                width: 400,
                height: 24,
            },
            "CreateWindowExW command button name edit",
        )?,
        executable_path_label: create_dialog_child(
            hwnd,
            instance,
            static_class.as_ptr(),
            tr(language, "실행 대상", "Executable"),
            label_style,
            0,
            RectSpec {
                x: 18,
                y: 62,
                width: 108,
                height: 22,
            },
            "CreateWindowExW executable path label",
        )?,
        executable_path_edit: create_dialog_child(
            hwnd,
            instance,
            edit_class.as_ptr(),
            "",
            edit_style,
            CONTROL_EXECUTABLE_PATH_EDIT_ID,
            RectSpec {
                x: 132,
                y: 60,
                width: 310,
                height: 24,
            },
            "CreateWindowExW executable path edit",
        )?,
        executable_browse_button: create_dialog_child(
            hwnd,
            instance,
            button_class.as_ptr(),
            tr(language, "찾기", "Browse"),
            button_style,
            CONTROL_EXECUTABLE_BROWSE_BUTTON_ID,
            RectSpec {
                x: 452,
                y: 59,
                width: 80,
                height: 26,
            },
            "CreateWindowExW executable browse button",
        )?,
        arguments_label: create_dialog_child(
            hwnd,
            instance,
            static_class.as_ptr(),
            tr(language, "인수", "Arguments"),
            label_style,
            0,
            RectSpec {
                x: 18,
                y: 104,
                width: 108,
                height: 22,
            },
            "CreateWindowExW arguments label",
        )?,
        arguments_edit: create_dialog_child(
            hwnd,
            instance,
            edit_class.as_ptr(),
            "",
            edit_style,
            CONTROL_ARGUMENTS_EDIT_ID,
            RectSpec {
                x: 132,
                y: 102,
                width: 400,
                height: 24,
            },
            "CreateWindowExW arguments edit",
        )?,
        token_path_button: create_dialog_child(
            hwnd,
            instance,
            button_class.as_ptr(),
            "{path}",
            button_style,
            CONTROL_TOKEN_PATH_BUTTON_ID,
            RectSpec {
                x: token_first_column_x,
                y: token_first_row_y,
                width: token_button_width,
                height: token_button_height,
            },
            "CreateWindowExW token path button",
        )?,
        token_name_button: create_dialog_child(
            hwnd,
            instance,
            button_class.as_ptr(),
            "{name}",
            button_style,
            CONTROL_TOKEN_NAME_BUTTON_ID,
            RectSpec {
                x: token_second_column_x,
                y: token_first_row_y,
                width: token_button_width,
                height: token_button_height,
            },
            "CreateWindowExW token name button",
        )?,
        token_select_file_button: create_dialog_child(
            hwnd,
            instance,
            button_class.as_ptr(),
            "{selectfile}",
            button_style,
            CONTROL_TOKEN_SELECT_FILE_BUTTON_ID,
            RectSpec {
                x: token_third_column_x,
                y: token_first_row_y,
                width: token_button_width,
                height: token_button_height,
            },
            "CreateWindowExW token select file button",
        )?,
        token_select_dir_button: create_dialog_child(
            hwnd,
            instance,
            button_class.as_ptr(),
            "{selectdir}",
            button_style,
            CONTROL_TOKEN_SELECT_DIR_BUTTON_ID,
            RectSpec {
                x: token_first_column_x,
                y: token_second_row_y,
                width: token_button_width,
                height: token_button_height,
            },
            "CreateWindowExW token select dir button",
        )?,
        token_input_text_button: create_dialog_child(
            hwnd,
            instance,
            button_class.as_ptr(),
            "{inputtext}",
            button_style,
            CONTROL_TOKEN_INPUT_TEXT_BUTTON_ID,
            RectSpec {
                x: token_second_column_x,
                y: token_second_row_y,
                width: token_button_width,
                height: token_button_height,
            },
            "CreateWindowExW token input text button",
        )?,
        token_language_button: create_dialog_child(
            hwnd,
            instance,
            button_class.as_ptr(),
            "{Language}",
            button_style,
            CONTROL_TOKEN_LANGUAGE_BUTTON_ID,
            RectSpec {
                x: token_third_column_x,
                y: token_second_row_y,
                width: token_button_width,
                height: token_button_height,
            },
            "CreateWindowExW token language button",
        )?,
        execution_type_label: create_dialog_child(
            hwnd,
            instance,
            static_class.as_ptr(),
            tr(language, "실행 방식", "Execution Type"),
            label_style,
            0,
            RectSpec {
                x: 18,
                y: execution_type_y,
                width: execution_type_label_width,
                height: 22,
            },
            "CreateWindowExW execution type label",
        )?,
        shell_api_radio: create_dialog_child(
            hwnd,
            instance,
            button_class.as_ptr(),
            tr(language, "직접", "Direct"),
            shell_radio_style,
            CONTROL_EXECUTION_TYPE_SHELL_RADIO_ID,
            RectSpec {
                x: execution_type_shell_x,
                y: execution_type_y - 2,
                width: 140,
                height: 24,
            },
            "CreateWindowExW shell api radio",
        )?,
        external_terminal_radio: create_dialog_child(
            hwnd,
            instance,
            button_class.as_ptr(),
            tr(language, "터미널", "Terminal"),
            radio_style,
            CONTROL_EXECUTION_TYPE_EXTERNAL_RADIO_ID,
            RectSpec {
                x: execution_type_external_x,
                y: execution_type_y - 2,
                width: 180,
                height: 24,
            },
            "CreateWindowExW external terminal radio",
        )?,
        ok_button: create_dialog_child(
            hwnd,
            instance,
            button_class.as_ptr(),
            tr(language, "저장", "Save"),
            ok_button_style,
            IDOK,
            RectSpec {
                x: 286,
                y: 268,
                width: 76,
                height: 28,
            },
            "CreateWindowExW command button ok button",
        )?,
        cancel_button: create_dialog_child(
            hwnd,
            instance,
            button_class.as_ptr(),
            tr(language, "취소", "Cancel"),
            button_style,
            IDCANCEL,
            RectSpec {
                x: 370,
                y: 268,
                width: 76,
                height: 28,
            },
            "CreateWindowExW command button cancel button",
        )?,
        apply_button: create_dialog_child(
            hwnd,
            instance,
            button_class.as_ptr(),
            tr(language, "적용", "Apply"),
            button_style,
            CONTROL_APPLY_BUTTON_ID,
            RectSpec {
                x: 454,
                y: 268,
                width: 76,
                height: 28,
            },
            "CreateWindowExW command button apply button",
        )?,
    };

    Ok(())
}

fn create_font_dialog_controls(
    hwnd: HWND,
    instance: HINSTANCE,
    context: &mut FontDialogContext,
) -> AppResult<()> {
    let static_class = wide_null("STATIC");
    let combo_class = wide_null("COMBOBOX");
    let button_class = wide_null("BUTTON");

    let label_style = WS_CHILD | WS_VISIBLE;
    let preview_style = WS_CHILD | WS_VISIBLE | WS_BORDER;
    let combo_style = WS_CHILD
        | WS_VISIBLE
        | WS_TABSTOP
        | WS_BORDER
        | CBS_DROPDOWNLIST as u32
        | CBS_HASSTRINGS as u32;
    let font_combo_style = combo_style | WS_VSCROLL;
    let button_style = WS_CHILD | WS_VISIBLE | WS_TABSTOP | BS_PUSHBUTTON as u32;
    let apply_button_style =
        WS_CHILD | WS_VISIBLE | WS_TABSTOP | WS_GROUP | BS_DEFPUSHBUTTON as u32;
    let language = context.language;

    context.controls = FontDialogControls {
        font_label: create_dialog_child(
            hwnd,
            instance,
            static_class.as_ptr(),
            tr(language, "글꼴", "Font"),
            label_style,
            0,
            RectSpec {
                x: 18,
                y: 20,
                width: 108,
                height: 22,
            },
            "CreateWindowExW font label",
        )?,
        font_combo: create_dialog_child(
            hwnd,
            instance,
            combo_class.as_ptr(),
            "",
            font_combo_style,
            CONTROL_FONT_FAMILY_COMBO_ID,
            RectSpec {
                x: 132,
                y: 18,
                width: 330,
                height: 210,
            },
            "CreateWindowExW font combo",
        )?,
        size_label: create_dialog_child(
            hwnd,
            instance,
            static_class.as_ptr(),
            tr(language, "크기", "Size"),
            label_style,
            0,
            RectSpec {
                x: 18,
                y: 62,
                width: 108,
                height: 22,
            },
            "CreateWindowExW font size label",
        )?,
        size_combo: create_dialog_child(
            hwnd,
            instance,
            combo_class.as_ptr(),
            "",
            combo_style,
            CONTROL_FONT_SIZE_COMBO_ID,
            RectSpec {
                x: 132,
                y: 60,
                width: 120,
                height: 180,
            },
            "CreateWindowExW font size combo",
        )?,
        preview_label: create_dialog_child(
            hwnd,
            instance,
            static_class.as_ptr(),
            font_preview_text(language),
            preview_style,
            0,
            RectSpec {
                x: 18,
                y: 106,
                width: 444,
                height: 68,
            },
            "CreateWindowExW font preview label",
        )?,
        default_button: create_dialog_child(
            hwnd,
            instance,
            button_class.as_ptr(),
            tr(language, "기본값", "Default"),
            button_style,
            CONTROL_FONT_DEFAULT_BUTTON_ID,
            RectSpec {
                x: 18,
                y: 202,
                width: 82,
                height: 28,
            },
            "CreateWindowExW font default button",
        )?,
        apply_button: create_dialog_child(
            hwnd,
            instance,
            button_class.as_ptr(),
            tr(language, "적용", "Apply"),
            apply_button_style,
            CONTROL_FONT_APPLY_BUTTON_ID,
            RectSpec {
                x: 298,
                y: 202,
                width: 78,
                height: 28,
            },
            "CreateWindowExW font apply button",
        )?,
        cancel_button: create_dialog_child(
            hwnd,
            instance,
            button_class.as_ptr(),
            tr(language, "취소", "Cancel"),
            button_style,
            IDCANCEL,
            RectSpec {
                x: 384,
                y: 202,
                width: 78,
                height: 28,
            },
            "CreateWindowExW font cancel button",
        )?,
    };

    Ok(())
}

fn create_text_input_dialog_controls(
    hwnd: HWND,
    instance: HINSTANCE,
    context: &mut TextInputDialogContext,
) -> AppResult<()> {
    let static_class = wide_null("STATIC");
    let edit_class = wide_null("EDIT");
    let button_class = wide_null("BUTTON");

    let label_style = WS_CHILD | WS_VISIBLE;
    let edit_style = WS_CHILD | WS_VISIBLE | WS_TABSTOP | ES_AUTOHSCROLL as u32;
    let button_style = WS_CHILD | WS_VISIBLE | WS_TABSTOP | BS_PUSHBUTTON as u32;
    let ok_button_style = WS_CHILD | WS_VISIBLE | WS_TABSTOP | WS_GROUP | BS_DEFPUSHBUTTON as u32;
    let language = context.language;

    context.controls = TextInputDialogControls {
        prompt_label: create_dialog_child(
            hwnd,
            instance,
            static_class.as_ptr(),
            &context.prompt,
            label_style,
            0,
            RectSpec {
                x: 18,
                y: 20,
                width: 336,
                height: 22,
            },
            "CreateWindowExW text input label",
        )?,
        value_edit: create_dialog_child_ex(
            hwnd,
            instance,
            WS_EX_CLIENTEDGE,
            edit_class.as_ptr(),
            "",
            edit_style,
            CONTROL_TEXT_INPUT_EDIT_ID,
            RectSpec {
                x: 18,
                y: 48,
                width: 336,
                height: 24,
            },
            "CreateWindowExW text input edit",
        )?,
        ok_button: create_dialog_child(
            hwnd,
            instance,
            button_class.as_ptr(),
            tr(language, "저장", "Save"),
            ok_button_style,
            IDOK,
            RectSpec {
                x: 198,
                y: 86,
                width: 74,
                height: 28,
            },
            "CreateWindowExW text input ok button",
        )?,
        cancel_button: create_dialog_child(
            hwnd,
            instance,
            button_class.as_ptr(),
            tr(language, "취소", "Cancel"),
            button_style,
            IDCANCEL,
            RectSpec {
                x: 280,
                y: 86,
                width: 74,
                height: 28,
            },
            "CreateWindowExW text input cancel button",
        )?,
    };

    Ok(())
}

fn create_about_dialog_controls(
    hwnd: HWND,
    instance: HINSTANCE,
    context: &mut AboutDialogContext,
    message: &str,
) -> AppResult<()> {
    let static_class = wide_null("STATIC");
    let button_class = wide_null("BUTTON");

    let label_style = WS_CHILD | WS_VISIBLE;
    let link_style = WS_CHILD | WS_VISIBLE | WS_TABSTOP | LWS_TRANSPARENT;
    let ok_button_style = WS_CHILD | WS_VISIBLE | WS_TABSTOP | WS_GROUP | BS_DEFPUSHBUTTON as u32;
    let language = context.language;

    context.controls = AboutDialogControls {
        message_label: create_dialog_child(
            hwnd,
            instance,
            static_class.as_ptr(),
            message,
            label_style,
            0,
            RectSpec {
                x: 24,
                y: 24,
                width: 248,
                height: 22,
            },
            "CreateWindowExW about message label",
        )?,
        repository_link: create_dialog_child(
            hwnd,
            instance,
            WC_LINK,
            &about_link_markup(APP_REPOSITORY_URL),
            link_style,
            CONTROL_ABOUT_LINK_ID,
            RectSpec {
                x: 24,
                y: 60,
                width: 248,
                height: 22,
            },
            "CreateWindowExW about repository link",
        )?,
        ok_button: create_dialog_child(
            hwnd,
            instance,
            button_class.as_ptr(),
            tr(language, "닫기", "Close"),
            ok_button_style,
            IDOK,
            RectSpec {
                x: 122,
                y: 88,
                width: 76,
                height: 28,
            },
            "CreateWindowExW about ok button",
        )?,
    };

    Ok(())
}

fn about_link_markup(url: &str) -> String {
    format!("<a href=\"{url}\">{url}</a>")
}

fn open_about_repository_link(hwnd: HWND, language: UiLanguage) {
    let operation = wide_null("open");
    let target = wide_null(APP_REPOSITORY_URL);

    // SAFETY: hwnd is the owner dialog, and the verb and URL are null-terminated strings that
    // remain alive for the duration of the platform ShellExecuteW call.
    let result = unsafe {
        ShellExecuteW(
            hwnd,
            operation.as_ptr(),
            target.as_ptr(),
            null(),
            null(),
            SW_SHOW,
        )
    };

    if (result as isize) <= 32 {
        show_about_link_open_error(hwnd, language);
    }
}

fn show_about_link_open_error(hwnd: HWND, language: UiLanguage) {
    let title = wide_null(tr(language, "정보", "About"));
    let message = wide_null(&format!(
        "{}\n{}",
        tr(
            language,
            "링크를 열 수 없습니다.",
            "Could not open the link."
        ),
        APP_REPOSITORY_URL
    ));

    // SAFETY: hwnd is the owner dialog and both strings are null-terminated for this call.
    unsafe {
        MessageBoxW(
            hwnd,
            message.as_ptr(),
            title.as_ptr(),
            MB_OK | MB_ICONWARNING,
        );
    }
}

fn create_language_config_dialog_controls(
    hwnd: HWND,
    instance: HINSTANCE,
    context: &mut LanguageConfigDialogContext,
) -> AppResult<()> {
    let static_class = wide_null("STATIC");
    let edit_class = wide_null("EDIT");
    let button_class = wide_null("BUTTON");

    let label_style = WS_CHILD | WS_VISIBLE;
    let edit_style = WS_CHILD
        | WS_VISIBLE
        | WS_TABSTOP
        | WS_BORDER
        | WS_VSCROLL
        | ES_MULTILINE as u32
        | ES_AUTOVSCROLL as u32
        | ES_WANTRETURN as u32;
    let button_style = WS_CHILD | WS_VISIBLE | WS_TABSTOP | BS_PUSHBUTTON as u32;
    let ok_button_style = WS_CHILD | WS_VISIBLE | WS_TABSTOP | WS_GROUP | BS_DEFPUSHBUTTON as u32;
    let language = context.language;

    context.controls = LanguageConfigDialogControls {
        languages_label: create_dialog_child(
            hwnd,
            instance,
            static_class.as_ptr(),
            tr(language, "언어 목록", "Language List"),
            label_style,
            0,
            RectSpec {
                x: 18,
                y: 18,
                width: 160,
                height: 22,
            },
            "CreateWindowExW language config label",
        )?,
        languages_edit: create_dialog_child_ex(
            hwnd,
            instance,
            WS_EX_CLIENTEDGE,
            edit_class.as_ptr(),
            "",
            edit_style,
            CONTROL_LANGUAGE_CONFIG_EDIT_ID,
            RectSpec {
                x: 18,
                y: 46,
                width: 404,
                height: 176,
            },
            "CreateWindowExW language config edit",
        )?,
        default_button: create_dialog_child(
            hwnd,
            instance,
            button_class.as_ptr(),
            tr(language, "기본값", "Default"),
            button_style,
            CONTROL_LANGUAGE_CONFIG_DEFAULT_BUTTON_ID,
            RectSpec {
                x: 18,
                y: 238,
                width: 82,
                height: 28,
            },
            "CreateWindowExW language config default button",
        )?,
        ok_button: create_dialog_child(
            hwnd,
            instance,
            button_class.as_ptr(),
            tr(language, "저장", "Save"),
            ok_button_style,
            IDOK,
            RectSpec {
                x: 266,
                y: 238,
                width: 74,
                height: 28,
            },
            "CreateWindowExW language config ok button",
        )?,
        cancel_button: create_dialog_child(
            hwnd,
            instance,
            button_class.as_ptr(),
            tr(language, "취소", "Cancel"),
            button_style,
            IDCANCEL,
            RectSpec {
                x: 348,
                y: 238,
                width: 74,
                height: 28,
            },
            "CreateWindowExW language config cancel button",
        )?,
    };

    Ok(())
}

// This helper intentionally follows CreateWindowExW's child-control parameter list.
#[allow(clippy::too_many_arguments)]
fn create_dialog_child(
    parent: HWND,
    instance: HINSTANCE,
    class_name: *const u16,
    text: &str,
    style: u32,
    control_id: i32,
    rect: RectSpec,
    operation: &'static str,
) -> AppResult<HWND> {
    create_dialog_child_ex(
        parent, instance, 0, class_name, text, style, control_id, rect, operation,
    )
}

// This helper intentionally follows CreateWindowExW's child-control parameter list.
#[allow(clippy::too_many_arguments)]
fn create_dialog_child_ex(
    parent: HWND,
    instance: HINSTANCE,
    extended_style: u32,
    class_name: *const u16,
    text: &str,
    style: u32,
    control_id: i32,
    rect: RectSpec,
    operation: &'static str,
) -> AppResult<HWND> {
    let text = wide_null(text);
    let rect = scale_rect_for_dialog(rect, dialog_font_size(parent), window_dpi(parent));
    // SAFETY: parent is a valid dialog window, class_name names a registered Win32 class, and
    // control_id is passed as HMENU per Win32 child-control convention.
    let hwnd = unsafe {
        CreateWindowExW(
            extended_style,
            class_name,
            text.as_ptr(),
            style,
            rect.x,
            rect.y,
            rect.width,
            rect.height,
            parent,
            control_id as usize as HMENU,
            instance,
            null(),
        )
    };

    if hwnd.is_null() {
        Err(last_error(operation))
    } else {
        Ok(hwnd)
    }
}

fn initialize_workspace_dialog_values(context: &mut WorkspaceDialogContext) {
    let fallback_language = default_workspace_language_for_options(&context.language_options);
    let selected_language = context
        .initial_workspace
        .as_ref()
        .map(|workspace| workspace.language.as_str())
        .unwrap_or(fallback_language.as_str());

    populate_language_combo(
        context.controls.language_combo,
        &context.language_options,
        selected_language,
    );

    if let Some(workspace) = context.initial_workspace.as_ref() {
        set_window_text(context.controls.name_edit, &workspace.name);
        set_window_text(context.controls.path_edit, &workspace.path);
    }
}

fn initialize_command_button_dialog_values(hwnd: HWND, context: &mut CommandButtonDialogContext) {
    let execution_type = context
        .initial_button
        .as_ref()
        .map(|button| button.execution_type)
        .unwrap_or(ExecutionType::ShellApi);

    if let Some(button) = context.initial_button.as_ref() {
        set_window_text(context.controls.button_name_edit, &button.button_name);
        set_window_text(
            context.controls.executable_path_edit,
            &button.executable_path,
        );
        set_window_text(context.controls.arguments_edit, &button.arguments);
    }

    let selected_radio = match execution_type {
        ExecutionType::ShellApi => CONTROL_EXECUTION_TYPE_SHELL_RADIO_ID,
        ExecutionType::ExternalTerminal => CONTROL_EXECUTION_TYPE_EXTERNAL_RADIO_ID,
    };

    // SAFETY: hwnd is this dialog window and the ID range identifies its two radio buttons.
    unsafe {
        CheckRadioButton(
            hwnd,
            CONTROL_EXECUTION_TYPE_SHELL_RADIO_ID,
            CONTROL_EXECUTION_TYPE_EXTERNAL_RADIO_ID,
            selected_radio,
        );
    }
}

fn initialize_font_dialog_values(hwnd: HWND, context: &mut FontDialogContext) {
    populate_font_combo(
        context.controls.font_combo,
        &context.fonts,
        &context.initial_view.font_family,
    );
    populate_font_size_combo(context.controls.size_combo, context.initial_view.font_size);
    font_dialog_update_preview(hwnd, context);
}

fn initialize_language_config_dialog_values(context: &mut LanguageConfigDialogContext) {
    set_window_text(
        context.controls.languages_edit,
        &context.initial_languages.join("\r\n"),
    );
}

fn initialize_text_input_dialog_values(context: &mut TextInputDialogContext) {
    set_window_text(context.controls.value_edit, &context.initial_value);
}

fn populate_language_combo(combo: HWND, language_options: &[String], selected_language: &str) {
    reset_combo(combo);

    let mut selected_index = None;
    for language in language_options {
        let index = combo_add_string(combo, language);
        if language.eq_ignore_ascii_case(selected_language) {
            selected_index = index;
        }
    }

    let index = selected_index.unwrap_or(0);
    // SAFETY: combo is a combo box control and index is one of the inserted items, or 0 when
    // insertion failed; Windows safely ignores an out-of-range selection.
    unsafe {
        SendMessageW(combo, CB_SETCURSEL, index as WPARAM, 0);
    }
}

fn populate_font_combo(combo: HWND, fonts: &[String], selected_font: &str) {
    reset_combo(combo);

    let mut selected_index = None;
    for font in fonts {
        let index = combo_add_string(combo, font);
        if font.eq_ignore_ascii_case(selected_font) {
            selected_index = index;
        }
    }

    let index = selected_index.unwrap_or(0);
    // SAFETY: combo is a combo box control. Windows safely ignores out-of-range selection.
    unsafe {
        SendMessageW(combo, CB_SETCURSEL, index as WPARAM, 0);
    }
}

fn populate_font_size_combo(combo: HWND, selected_font_size: u16) {
    reset_combo(combo);

    let selected_font_size = normalize_ui_font_size(selected_font_size);
    let mut selected_index = None;
    for font_size in UI_FONT_SIZE_OPTIONS {
        let label = font_size.to_string();
        let index = combo_add_string(combo, &label);
        if *font_size == selected_font_size {
            selected_index = index;
        }
    }

    let index = selected_index.unwrap_or(0);
    // SAFETY: combo is a combo box control and index is one of the inserted items, or 0 when
    // insertion failed; Windows safely ignores an out-of-range selection.
    unsafe {
        SendMessageW(combo, CB_SETCURSEL, index as WPARAM, 0);
    }
}

fn reset_combo(combo: HWND) {
    // SAFETY: combo is a combo box control.
    unsafe {
        SendMessageW(combo, CB_RESETCONTENT, 0, 0);
    }
}

fn combo_add_string(combo: HWND, text: &str) -> Option<usize> {
    let text = wide_null(text);
    // SAFETY: combo is a combo box control and text is a valid null-terminated UTF-16 string.
    let result = unsafe { SendMessageW(combo, CB_ADDSTRING, 0, text.as_ptr() as LPARAM) };
    if result < 0 {
        None
    } else {
        usize::try_from(result).ok()
    }
}

fn select_combo_text(combo: HWND, text: &str) -> bool {
    let text = wide_null(text);
    // SAFETY: combo is a combo box control. Starting at -1 searches from the beginning.
    let index = unsafe {
        SendMessageW(
            combo,
            CB_FINDSTRINGEXACT,
            usize::MAX,
            text.as_ptr() as LPARAM,
        )
    };
    if index == CB_ERR as isize || index < 0 {
        return false;
    }

    // SAFETY: index was returned by CB_FINDSTRINGEXACT for this combo box.
    unsafe {
        SendMessageW(combo, CB_SETCURSEL, index as WPARAM, 0);
    }
    true
}

fn modal_dialog_message_loop(hwnd: HWND, error_context: &'static str) -> AppResult<()> {
    let mut message = MSG::default();

    loop {
        // Keep this loop detached from the Rust dialog context so Win32 callbacks can mutate that
        // context without aliasing a live &mut.
        // SAFETY: hwnd is the dialog handle created for this modal loop. IsWindow only validates
        // whether the handle still refers to a live window.
        let is_open = unsafe { IsWindow(hwnd) };
        if is_open == 0 {
            break;
        }

        // SAFETY: message points to writable storage and a null HWND receives all thread messages.
        let result = unsafe { GetMessageW(&mut message, null_mut(), 0, 0) };
        if result == -1 {
            return Err(last_error(error_context));
        }

        if result == 0 {
            // Preserve the app shutdown signal for the outer message loop.
            // SAFETY: Re-posting WM_QUIT keeps the normal shutdown path intact.
            unsafe {
                PostQuitMessage(0);
            }
            break;
        }

        // SAFETY: hwnd is the dialog window and message was just returned by GetMessageW.
        let handled = unsafe { IsDialogMessageW(hwnd, &message) };
        if handled == 0 {
            // SAFETY: message contains a message just returned by GetMessageW.
            unsafe {
                TranslateMessage(&message);
                DispatchMessageW(&message);
            }
        }
    }

    Ok(())
}

fn workspace_dialog_try_accept(hwnd: HWND, context: &mut WorkspaceDialogContext) {
    if context.pending_path_validation.is_some() {
        return;
    }

    let path = window_text(context.controls.path_edit);
    let name = window_text(context.controls.name_edit);
    let language = combo_selected_text(context.controls.language_combo);

    if path.trim().is_empty() {
        show_warning_message(
            hwnd,
            tr(context.language, "워크스페이스", "Workspace"),
            tr(context.language, "폴더를 선택하세요.", "Select a folder."),
        );
        return;
    }

    workspace_dialog_begin_path_validation(hwnd, context, path, name, language);
}

fn workspace_dialog_begin_path_validation(
    hwnd: HWND,
    context: &mut WorkspaceDialogContext,
    path: String,
    name: String,
    language: Option<String>,
) {
    let request_id = next_workspace_dialog_request_id();
    context.pending_path_validation = Some(PendingWorkspacePathValidation {
        request_id,
        path: path.clone(),
        name,
        language,
    });
    workspace_dialog_set_path_validation_pending(context, true);

    let hwnd_value = hwnd as isize;
    let spawn_result = std::thread::Builder::new()
        .name("workspace-path-check".to_owned())
        .spawn(move || {
            let accessible = is_accessible_folder(&path);
            // SAFETY: hwnd_value is a window handle captured as an integer. The worker only posts a
            // message and never dereferences dialog state.
            unsafe {
                PostMessageW(
                    hwnd_value as HWND,
                    WM_WORKSPACE_DIALOG_PATH_CHECKED,
                    request_id as WPARAM,
                    if accessible { 1 } else { 0 },
                );
            }
        });

    if spawn_result.is_err() {
        context.pending_path_validation = None;
        workspace_dialog_set_path_validation_pending(context, false);
        show_warning_message(
            hwnd,
            tr(context.language, "워크스페이스", "Workspace"),
            tr(
                context.language,
                "폴더를 확인할 수 없습니다.",
                "Could not validate the folder.",
            ),
        );
    }
}

fn workspace_dialog_complete_path_validation(
    hwnd: HWND,
    context: &mut WorkspaceDialogContext,
    request_id: u32,
    accessible: bool,
) {
    let Some(pending) = context.pending_path_validation.take() else {
        return;
    };

    if pending.request_id != request_id {
        context.pending_path_validation = Some(pending);
        return;
    }

    workspace_dialog_set_path_validation_pending(context, false);
    if !accessible {
        show_warning_message(
            hwnd,
            tr(context.language, "워크스페이스", "Workspace"),
            tr(
                context.language,
                "접근 가능한 폴더를 선택하세요.",
                "Select an accessible folder.",
            ),
        );
        return;
    }

    workspace_dialog_finish_accept(hwnd, context, pending.path, pending.name, pending.language);
}

fn workspace_dialog_finish_accept(
    hwnd: HWND,
    context: &mut WorkspaceDialogContext,
    path: String,
    name: String,
    language: Option<String>,
) {
    if name.trim().is_empty() {
        show_warning_message(
            hwnd,
            tr(context.language, "워크스페이스", "Workspace"),
            tr(context.language, "이름을 입력하세요.", "Enter a name."),
        );
        return;
    }

    let Some(language) = language.filter(|value| !value.trim().is_empty()) else {
        show_warning_message(
            hwnd,
            tr(context.language, "워크스페이스", "Workspace"),
            tr(context.language, "언어를 선택하세요.", "Select a language."),
        );
        return;
    };

    if context
        .reserved_paths
        .iter()
        .any(|reserved| workspace_paths_equal(reserved, &path))
    {
        show_warning_message(
            hwnd,
            tr(context.language, "워크스페이스", "Workspace"),
            tr(
                context.language,
                "이미 등록된 폴더입니다.",
                "This folder is already registered.",
            ),
        );
        return;
    }

    match Workspace::new_with_language_options(path, name, language, &context.language_options) {
        Ok(workspace) => {
            context.result = Some(workspace);
            // SAFETY: hwnd is the dialog window for this context.
            unsafe {
                DestroyWindow(hwnd);
            }
        }
        Err(error) => show_warning_message(
            hwnd,
            tr(context.language, "워크스페이스", "Workspace"),
            &localized_domain_error(context.language, &error),
        ),
    }
}

fn next_workspace_dialog_request_id() -> u32 {
    NEXT_WORKSPACE_DIALOG_REQUEST_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
}

fn workspace_dialog_set_path_validation_pending(
    context: &mut WorkspaceDialogContext,
    pending: bool,
) {
    let enabled = !pending;
    set_control_enabled(context.controls.name_edit, enabled);
    set_control_enabled(context.controls.path_edit, enabled);
    set_control_enabled(context.controls.browse_button, enabled);
    set_control_enabled(context.controls.language_combo, enabled);
    workspace_dialog_update_ok_enabled(context);
}

fn workspace_dialog_update_ok_enabled(context: &WorkspaceDialogContext) {
    let enabled =
        context.pending_path_validation.is_none() && context.pending_language_inference.is_none();
    set_control_enabled(context.controls.ok_button, enabled);
}

fn set_control_enabled(hwnd: HWND, enabled: bool) {
    if hwnd.is_null() {
        return;
    }

    // SAFETY: hwnd is a dialog control handle owned by this UI thread.
    unsafe {
        EnableWindow(hwnd, if enabled { 1 } else { 0 });
    }
}

fn command_button_dialog_try_accept(hwnd: HWND, context: &mut CommandButtonDialogContext) {
    if command_button_dialog_try_apply(hwnd, context) {
        // SAFETY: hwnd is the dialog window for this context.
        unsafe {
            DestroyWindow(hwnd);
        }
    }
}

fn font_dialog_try_accept(hwnd: HWND, context: &mut FontDialogContext) {
    let view = font_dialog_current_view(context);
    if !font_family_available(&context.fonts, &view.font_family) {
        show_warning_message(
            hwnd,
            tr(context.language, "글꼴", "Font"),
            tr(
                context.language,
                "목록에서 글꼴을 선택하세요.",
                "Select a font from the list.",
            ),
        );
        return;
    }

    context.result = Some(view);
    // SAFETY: hwnd is the dialog window for this context.
    unsafe {
        DestroyWindow(hwnd);
    }
}

fn font_dialog_reset_default(hwnd: HWND, context: &mut FontDialogContext) {
    let default = ViewSettings::default();
    if !select_combo_text(context.controls.font_combo, &default.font_family) {
        // SAFETY: font_combo is a combo box control; selecting index 0 is ignored if empty.
        unsafe {
            SendMessageW(context.controls.font_combo, CB_SETCURSEL, 0, 0);
        }
    }
    select_combo_text(
        context.controls.size_combo,
        &normalize_ui_font_size(default.font_size).to_string(),
    );
    font_dialog_update_preview(hwnd, context);
}

fn font_dialog_update_preview(hwnd: HWND, context: &mut FontDialogContext) {
    let view = font_dialog_current_view(context);
    context.preview_font = UiFont::from_view(&view, context.dpi);
    set_window_text(
        context.controls.preview_label,
        font_preview_text(context.language),
    );
    apply_font_to_handles(
        &[context.controls.preview_label],
        context.preview_font.handle(),
    );

    // SAFETY: hwnd is the dialog window and preview_label is its child control.
    unsafe {
        InvalidateRect(context.controls.preview_label, null(), 1);
        InvalidateRect(hwnd, null(), 1);
    }
}

fn font_dialog_current_view(context: &FontDialogContext) -> ViewSettings {
    let font_family = combo_selected_text(context.controls.font_combo)
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| context.initial_view.font_family.clone());
    let font_size = combo_selected_text(context.controls.size_combo)
        .and_then(|value| value.parse::<u16>().ok())
        .unwrap_or(context.initial_view.font_size);

    ViewSettings::new(font_family, font_size, context.initial_view.theme.clone())
        .with_ui_language(&context.initial_view.ui_language)
        .with_window_layout_from(&context.initial_view)
}

fn language_config_dialog_try_accept(hwnd: HWND, context: &mut LanguageConfigDialogContext) {
    let raw = window_text(context.controls.languages_edit);
    match parse_language_config_editor_text(&raw) {
        Ok(languages) => {
            context.result = Some(languages);
            // SAFETY: hwnd is the dialog window for this context.
            unsafe {
                DestroyWindow(hwnd);
            }
        }
        Err(error) => show_warning_message(
            hwnd,
            tr(context.language, "언어", "Language"),
            &localized_domain_error(context.language, &error),
        ),
    }
}

fn language_config_dialog_reset_default(context: &mut LanguageConfigDialogContext) {
    let text = default_workspace_language_options().join("\r\n");
    set_window_text(context.controls.languages_edit, &text);
}

fn parse_language_config_editor_text(
    text: &str,
) -> Result<Vec<String>, crate::domain::DomainValidationError> {
    normalize_workspace_language_options(text.split(['\r', '\n', ',']))
}

fn command_button_dialog_try_apply(hwnd: HWND, context: &mut CommandButtonDialogContext) -> bool {
    let Some(button) = command_button_from_dialog(hwnd, context) else {
        return false;
    };

    // SAFETY: app_context is supplied by the main window handler and remains alive while this
    // modal dialog is running.
    let Some(app_context) = (unsafe { context.app_context.as_mut() }) else {
        show_error_message(
            hwnd,
            tr(context.language, "명령", "Command"),
            tr(
                context.language,
                "앱 상태를 찾을 수 없습니다.",
                "Could not find the app state.",
            ),
        );
        return false;
    };

    if let Some(button_index) = context.button_index {
        match app_context.spec.state.command_button_matches(
            context.tab_index,
            button_index,
            &button,
        ) {
            Ok(true) => return true,
            Ok(false) => {}
            Err(error) => {
                show_error_message(
                    hwnd,
                    tr(context.language, "명령", "Command"),
                    &localized_command_button_mutation_error(context.language, &error),
                );
                return false;
            }
        }
    }

    let previous_state = app_context.spec.state.clone();
    let previous_button_index = context.button_index;
    let result = match (context.mode, context.button_index) {
        (CommandButtonDialogMode::Add, None) => {
            match app_context
                .spec
                .state
                .add_command_button(context.tab_index, button)
            {
                Ok(index) => {
                    context.button_index = Some(index);
                    Ok(())
                }
                Err(error) => Err(error),
            }
        }
        (_, Some(button_index)) => {
            app_context
                .spec
                .state
                .update_command_button(context.tab_index, button_index, button)
        }
        (CommandButtonDialogMode::Edit, None) => {
            Err(crate::domain::CommandButtonMutationError::InvalidButtonIndex(0))
        }
    };

    match result {
        Ok(()) => {
            if !persist_settings_or_restore(
                context.owner,
                &mut app_context.spec.state,
                previous_state,
            ) {
                context.button_index = previous_button_index;
                return false;
            }
            refresh_command_buttons(context.owner, app_context);
            update_commands_menu_state(context.owner, app_context);
            true
        }
        Err(error) => {
            show_error_message(
                hwnd,
                tr(context.language, "명령", "Command"),
                &localized_command_button_mutation_error(context.language, &error),
            );
            false
        }
    }
}

fn command_button_from_dialog(
    hwnd: HWND,
    context: &CommandButtonDialogContext,
) -> Option<CommandButton> {
    let button_name = window_text(context.controls.button_name_edit);
    let executable_path = window_text(context.controls.executable_path_edit);
    let arguments = window_text(context.controls.arguments_edit);

    if button_name.trim().is_empty() {
        show_warning_message(
            hwnd,
            tr(context.language, "명령", "Command"),
            tr(context.language, "이름을 입력하세요.", "Enter a name."),
        );
        return None;
    }

    if executable_path.trim().is_empty() {
        show_warning_message(
            hwnd,
            tr(context.language, "명령", "Command"),
            tr(
                context.language,
                "실행 대상을 입력하세요.",
                "Enter an executable target.",
            ),
        );
        return None;
    }

    let unknown_tokens = unknown_argument_tokens(&arguments);
    if !unknown_tokens.is_empty() {
        let message = crate::domain::ArgumentResolutionError::UnknownTokens(unknown_tokens)
            .user_message_for_language(context.language);
        show_warning_message(hwnd, tr(context.language, "명령", "Command"), &message);
        return None;
    }

    let Some(execution_type) = selected_execution_type(context) else {
        show_warning_message(
            hwnd,
            tr(context.language, "명령", "Command"),
            tr(
                context.language,
                "실행 방식을 선택하세요.",
                "Select an execution type.",
            ),
        );
        return None;
    };

    match CommandButton::new(button_name, executable_path, arguments, execution_type) {
        Ok(button) => Some(button),
        Err(error) => {
            show_warning_message(
                hwnd,
                tr(context.language, "명령", "Command"),
                &localized_domain_error(context.language, &error),
            );
            None
        }
    }
}

fn selected_execution_type(context: &CommandButtonDialogContext) -> Option<ExecutionType> {
    // SAFETY: radio handles are child buttons created by this dialog.
    let shell_checked =
        unsafe { SendMessageW(context.controls.shell_api_radio, BM_GETCHECK, 0, 0) };
    if shell_checked == BST_CHECKED as isize {
        return Some(ExecutionType::ShellApi);
    }

    // SAFETY: radio handles are child buttons created by this dialog.
    let external_checked =
        unsafe { SendMessageW(context.controls.external_terminal_radio, BM_GETCHECK, 0, 0) };
    if external_checked == BST_CHECKED as isize {
        Some(ExecutionType::ExternalTerminal)
    } else {
        None
    }
}

fn command_button_dialog_browse_executable(hwnd: HWND, context: &mut CommandButtonDialogContext) {
    let initial_path = window_text(context.controls.executable_path_edit);
    let Some(path) = browse_for_executable_file(hwnd, &initial_path, context.language) else {
        return;
    };

    set_window_text(
        context.controls.executable_path_edit,
        &path.display().to_string(),
    );
}

fn command_button_dialog_insert_argument_token(
    _hwnd: HWND,
    context: &mut CommandButtonDialogContext,
    token: &str,
) {
    let token = wide_null(token);
    // SAFETY: arguments_edit is an EDIT control. EM_REPLACESEL inserts at the current selection
    // or caret position, and SetFocus restores keyboard focus to the arguments field.
    unsafe {
        SendMessageW(
            context.controls.arguments_edit,
            EM_REPLACESEL,
            1,
            token.as_ptr() as LPARAM,
        );
        SetFocus(context.controls.arguments_edit);
    }
}

fn argument_token_for_button_id(control_id: u32) -> Option<&'static str> {
    match control_id as i32 {
        CONTROL_TOKEN_PATH_BUTTON_ID => ARGUMENT_TOKENS.first().copied(),
        CONTROL_TOKEN_NAME_BUTTON_ID => ARGUMENT_TOKENS.get(1).copied(),
        CONTROL_TOKEN_SELECT_FILE_BUTTON_ID => ARGUMENT_TOKENS.get(2).copied(),
        CONTROL_TOKEN_SELECT_DIR_BUTTON_ID => ARGUMENT_TOKENS.get(3).copied(),
        CONTROL_TOKEN_INPUT_TEXT_BUTTON_ID => ARGUMENT_TOKENS.get(4).copied(),
        CONTROL_TOKEN_LANGUAGE_BUTTON_ID => ARGUMENT_TOKENS.get(5).copied(),
        _ => None,
    }
}

fn browse_for_executable_file(
    owner: HWND,
    initial_path: &str,
    language: UiLanguage,
) -> Option<PathBuf> {
    let mut file_buffer = vec![0u16; 32768];
    let initial_path = initial_path.trim();
    if !initial_path.is_empty() {
        let initial = initial_path.encode_utf16().collect::<Vec<_>>();
        let copy_len = initial.len().min(file_buffer.len().saturating_sub(1));
        file_buffer[..copy_len].copy_from_slice(&initial[..copy_len]);
    }

    let filter = wide_filter(&[
        tr(language, "실행 파일", "Executable Files"),
        "*.exe;*.cmd;*.bat;*.ps1;*.com",
        tr(language, "모든 파일", "All Files"),
        "*.*",
    ]);
    let title = wide_null(tr(language, "실행 대상 선택", "Select Executable"));
    let mut open_file = OPENFILENAMEW {
        lStructSize: size_of::<OPENFILENAMEW>() as u32,
        hwndOwner: owner,
        lpstrFilter: filter.as_ptr(),
        lpstrFile: file_buffer.as_mut_ptr(),
        nMaxFile: file_buffer.len() as u32,
        lpstrTitle: title.as_ptr(),
        Flags: OFN_EXPLORER | OFN_FILEMUSTEXIST | OFN_PATHMUSTEXIST,
        ..OPENFILENAMEW::default()
    };

    // SAFETY: open_file points to initialized OPENFILENAMEW storage and file_buffer is writable
    // for nMaxFile UTF-16 code units for the duration of the call.
    let ok = unsafe { GetOpenFileNameW(&mut open_file) };
    if ok == 0 {
        None
    } else {
        Some(PathBuf::from(wide_null_buffer_to_string(&file_buffer)))
    }
}

fn text_input_dialog_try_accept(hwnd: HWND, context: &mut TextInputDialogContext) {
    let value = window_text(context.controls.value_edit);
    if !context.allow_empty && value.trim().is_empty() {
        show_warning_message(
            hwnd,
            &context.title,
            tr(context.language, "이름을 입력하세요.", "Enter a name."),
        );
        return;
    }

    context.result = if context.allow_empty {
        Some(value)
    } else {
        Some(value.trim().to_owned())
    };
    // SAFETY: hwnd is the dialog window for this context.
    unsafe {
        DestroyWindow(hwnd);
    }
}

fn workspace_dialog_browse_folder(hwnd: HWND, context: &mut WorkspaceDialogContext) {
    let Some(path) = browse_for_folder(hwnd, tr(context.language, "폴더 선택", "Select Folder"))
    else {
        return;
    };

    let path_text = path.display().to_string();
    context.pending_language_inference = None;
    set_window_text(context.controls.path_edit, &path_text);

    if context.mode == WorkspaceDialogMode::Add {
        let folder_name = default_workspace_name_for_path(&path);
        let current_name = window_text(context.controls.name_edit);
        let should_replace_name = current_name.trim().is_empty()
            || context
                .previous_folder_default_name
                .as_ref()
                .is_some_and(|previous| previous == &current_name);

        if should_replace_name {
            set_window_text(context.controls.name_edit, &folder_name);
        }

        context.previous_folder_default_name = Some(folder_name);

        workspace_dialog_begin_language_inference(hwnd, context, path);
    }
}

fn workspace_dialog_begin_language_inference(
    hwnd: HWND,
    context: &mut WorkspaceDialogContext,
    path: PathBuf,
) {
    let request_id = next_workspace_dialog_request_id();
    context.pending_language_inference = Some(request_id);
    workspace_dialog_update_ok_enabled(context);

    let hwnd_value = hwnd as isize;
    let language_options = context.language_options.clone();
    let spawn_result = std::thread::Builder::new()
        .name("workspace-language-infer".to_owned())
        .spawn(move || {
            let language_index = infer_workspace_language_from_folder(&path)
                .and_then(|language| {
                    language_options
                        .iter()
                        .position(|candidate| candidate.eq_ignore_ascii_case(language))
                })
                .and_then(|index| index.checked_add(1))
                .unwrap_or(0);

            // SAFETY: hwnd_value is a window handle captured as an integer. The worker only posts a
            // message and never dereferences dialog state.
            unsafe {
                PostMessageW(
                    hwnd_value as HWND,
                    WM_WORKSPACE_DIALOG_LANGUAGE_INFERRED,
                    request_id as WPARAM,
                    language_index as LPARAM,
                );
            }
        });

    if spawn_result.is_err() {
        context.pending_language_inference = None;
        workspace_dialog_update_ok_enabled(context);
    }
}

fn workspace_dialog_complete_language_inference(
    context: &mut WorkspaceDialogContext,
    request_id: u32,
    language_index: usize,
) {
    if context.pending_language_inference != Some(request_id) {
        return;
    }

    context.pending_language_inference = None;
    if language_index > 0
        && let Some(language) = context.language_options.get(language_index - 1)
    {
        select_combo_text(context.controls.language_combo, language);
    }
    workspace_dialog_update_ok_enabled(context);
}

fn workspace_dialog_cancel_language_inference(context: &mut WorkspaceDialogContext) {
    if context.pending_language_inference.take().is_some() {
        workspace_dialog_update_ok_enabled(context);
    }
}

fn combo_selected_text(combo: HWND) -> Option<String> {
    // SAFETY: combo is a combo box control.
    let selected = unsafe { SendMessageW(combo, CB_GETCURSEL, 0, 0) };
    if selected == CB_ERR as isize {
        return None;
    }

    // SAFETY: combo is a combo box control and selected is its current selection index.
    let len = unsafe { SendMessageW(combo, CB_GETLBTEXTLEN, selected as WPARAM, 0) };
    if len == CB_ERR as isize || len < 0 {
        return None;
    }

    let mut buffer = vec![0u16; len as usize + 1];
    // SAFETY: buffer has len + 1 writable u16 slots and selected is the current item index.
    let copied = unsafe {
        SendMessageW(
            combo,
            CB_GETLBTEXT,
            selected as WPARAM,
            buffer.as_mut_ptr() as LPARAM,
        )
    };
    if copied == CB_ERR as isize || copied < 0 {
        None
    } else {
        Some(String::from_utf16_lossy(&buffer[..copied as usize]))
    }
}

fn window_text(hwnd: HWND) -> String {
    // SAFETY: hwnd is an edit control created by this module.
    let len = unsafe { GetWindowTextLengthW(hwnd) };
    if len <= 0 {
        return String::new();
    }

    let mut buffer = vec![0u16; len as usize + 1];
    // SAFETY: buffer has len + 1 writable u16 slots, as required by GetWindowTextW.
    let copied = unsafe { GetWindowTextW(hwnd, buffer.as_mut_ptr(), buffer.len() as i32) };
    if copied <= 0 {
        String::new()
    } else {
        String::from_utf16_lossy(&buffer[..copied as usize])
    }
}

fn set_window_text(hwnd: HWND, text: &str) {
    let text = wide_null(text);
    // SAFETY: hwnd is a window/control created by this module and text is null-terminated.
    unsafe {
        SetWindowTextW(hwnd, text.as_ptr());
    }
}

fn centered_window_position(owner: HWND, width: i32, height: i32) -> (i32, i32) {
    let mut rect = RECT::default();
    // SAFETY: owner is the main window. On failure, GetWindowRect returns zero and rect remains 0.
    let ok = unsafe { GetWindowRect(owner, &mut rect) };
    if ok == 0 {
        return (CW_USEDEFAULT, CW_USEDEFAULT);
    }

    let owner_width = rect.right - rect.left;
    let owner_height = rect.bottom - rect.top;
    (
        rect.left + (owner_width - width).max(0) / 2,
        rect.top + (owner_height - height).max(0) / 2,
    )
}

unsafe extern "system" fn workspace_dialog_proc(
    hwnd: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match message {
        WM_NCCREATE => {
            // SAFETY: For WM_NCCREATE, lparam is a CREATESTRUCTW pointer supplied by Windows.
            let create = unsafe { &*(lparam as *const CREATESTRUCTW) };
            let context = create.lpCreateParams as *mut WorkspaceDialogContext;
            // SAFETY: The context pointer is supplied by show_workspace_dialog and remains valid
            // until WM_NCDESTROY clears GWLP_USERDATA. DefWindowProcW keeps default non-client
            // initialization, including applying the window caption supplied to CreateWindowExW.
            unsafe {
                SetWindowLongPtrW(hwnd, GWLP_USERDATA, context as isize);
                DefWindowProcW(hwnd, message, wparam, lparam)
            }
        }
        WM_WORKSPACE_DIALOG_PATH_CHECKED => {
            // SAFETY: The userdata pointer is set during WM_NCCREATE and cleared at WM_NCDESTROY.
            if let Some(context) = unsafe { workspace_dialog_context_mut(hwnd) }
                && let Ok(request_id) = u32::try_from(wparam)
            {
                workspace_dialog_complete_path_validation(hwnd, context, request_id, lparam != 0);
            }
            0
        }
        WM_WORKSPACE_DIALOG_LANGUAGE_INFERRED => {
            // SAFETY: The userdata pointer is set during WM_NCCREATE and cleared at WM_NCDESTROY.
            if let Some(context) = unsafe { workspace_dialog_context_mut(hwnd) }
                && let Ok(request_id) = u32::try_from(wparam)
                && let Ok(language_index) = usize::try_from(lparam)
            {
                workspace_dialog_complete_language_inference(context, request_id, language_index);
            }
            0
        }
        WM_COMMAND => {
            // SAFETY: The userdata pointer is set during WM_NCCREATE and cleared at WM_NCDESTROY.
            if let Some(context) = unsafe { workspace_dialog_context_mut(hwnd) } {
                match low_word(wparam) {
                    id if id == CONTROL_LANGUAGE_COMBO_ID as u32
                        && high_word(wparam) == CBN_SELCHANGE =>
                    {
                        workspace_dialog_cancel_language_inference(context);
                    }
                    id if id == CONTROL_BROWSE_BUTTON_ID as u32 => {
                        workspace_dialog_browse_folder(hwnd, context);
                    }
                    id if id == IDOK as u32 => {
                        workspace_dialog_try_accept(hwnd, context);
                    }
                    id if id == IDCANCEL as u32 => {
                        // SAFETY: hwnd is the dialog window.
                        unsafe {
                            DestroyWindow(hwnd);
                        }
                    }
                    _ => {}
                }
            }
            0
        }
        WM_CLOSE => {
            // SAFETY: hwnd is the dialog window.
            unsafe {
                DestroyWindow(hwnd);
            }
            0
        }
        WM_NCDESTROY => {
            // SAFETY: Clear the non-owning context pointer before the window handle dies.
            unsafe {
                SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0);
                DefWindowProcW(hwnd, message, wparam, lparam)
            }
        }
        _ => {
            // SAFETY: All messages not handled by this window procedure are delegated to Windows.
            unsafe { DefWindowProcW(hwnd, message, wparam, lparam) }
        }
    }
}

unsafe extern "system" fn command_button_dialog_proc(
    hwnd: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match message {
        WM_NCCREATE => {
            // SAFETY: For WM_NCCREATE, lparam is a CREATESTRUCTW pointer supplied by Windows.
            let create = unsafe { &*(lparam as *const CREATESTRUCTW) };
            let context = create.lpCreateParams as *mut CommandButtonDialogContext;
            // SAFETY: The context pointer is supplied by show_command_button_dialog and remains
            // valid until WM_NCDESTROY clears GWLP_USERDATA. DefWindowProcW keeps default
            // non-client initialization, including applying the window caption supplied to
            // CreateWindowExW.
            unsafe {
                SetWindowLongPtrW(hwnd, GWLP_USERDATA, context as isize);
                DefWindowProcW(hwnd, message, wparam, lparam)
            }
        }
        WM_COMMAND => {
            // SAFETY: The userdata pointer is set during WM_NCCREATE and cleared at WM_NCDESTROY.
            if let Some(context) = unsafe { command_button_dialog_context_mut(hwnd) } {
                let command_id = low_word(wparam);
                if let Some(token) = argument_token_for_button_id(command_id) {
                    command_button_dialog_insert_argument_token(hwnd, context, token);
                    return 0;
                }

                match command_id {
                    id if id == CONTROL_EXECUTABLE_BROWSE_BUTTON_ID as u32 => {
                        command_button_dialog_browse_executable(hwnd, context);
                    }
                    id if id == CONTROL_EXECUTION_TYPE_SHELL_RADIO_ID as u32
                        || id == CONTROL_EXECUTION_TYPE_EXTERNAL_RADIO_ID as u32 =>
                    {
                        // SAFETY: hwnd is this dialog window and both IDs are radio buttons in
                        // one contiguous group.
                        unsafe {
                            CheckRadioButton(
                                hwnd,
                                CONTROL_EXECUTION_TYPE_SHELL_RADIO_ID,
                                CONTROL_EXECUTION_TYPE_EXTERNAL_RADIO_ID,
                                command_id as i32,
                            );
                        }
                    }
                    id if id == IDOK as u32 => {
                        command_button_dialog_try_accept(hwnd, context);
                    }
                    id if id == IDCANCEL as u32 => {
                        // SAFETY: hwnd is the dialog window.
                        unsafe {
                            DestroyWindow(hwnd);
                        }
                    }
                    id if id == CONTROL_APPLY_BUTTON_ID as u32 => {
                        command_button_dialog_try_apply(hwnd, context);
                    }
                    _ => {}
                }
            }
            0
        }
        WM_CLOSE => {
            // SAFETY: hwnd is the dialog window.
            unsafe {
                DestroyWindow(hwnd);
            }
            0
        }
        WM_NCDESTROY => {
            // SAFETY: Clear the non-owning context pointer before the window handle dies.
            unsafe {
                SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0);
                DefWindowProcW(hwnd, message, wparam, lparam)
            }
        }
        _ => {
            // SAFETY: All messages not handled by this window procedure are delegated to Windows.
            unsafe { DefWindowProcW(hwnd, message, wparam, lparam) }
        }
    }
}

unsafe extern "system" fn font_dialog_proc(
    hwnd: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match message {
        WM_NCCREATE => {
            // SAFETY: For WM_NCCREATE, lparam is a CREATESTRUCTW pointer supplied by Windows.
            let create = unsafe { &*(lparam as *const CREATESTRUCTW) };
            let context = create.lpCreateParams as *mut FontDialogContext;
            // SAFETY: The context pointer is supplied by show_font_dialog and remains valid until
            // WM_NCDESTROY clears GWLP_USERDATA. DefWindowProcW keeps default non-client
            // initialization, including applying the window caption supplied to CreateWindowExW.
            unsafe {
                SetWindowLongPtrW(hwnd, GWLP_USERDATA, context as isize);
                DefWindowProcW(hwnd, message, wparam, lparam)
            }
        }
        WM_COMMAND => {
            // SAFETY: The userdata pointer is set during WM_NCCREATE and cleared at WM_NCDESTROY.
            if let Some(context) = unsafe { font_dialog_context_mut(hwnd) } {
                let command_id = low_word(wparam);
                let notification = high_word(wparam);
                if notification == CBN_SELCHANGE
                    && (command_id == CONTROL_FONT_FAMILY_COMBO_ID as u32
                        || command_id == CONTROL_FONT_SIZE_COMBO_ID as u32)
                {
                    font_dialog_update_preview(hwnd, context);
                    return 0;
                }

                match command_id {
                    id if id == CONTROL_FONT_DEFAULT_BUTTON_ID as u32 => {
                        font_dialog_reset_default(hwnd, context);
                    }
                    id if id == CONTROL_FONT_APPLY_BUTTON_ID as u32 => {
                        font_dialog_try_accept(hwnd, context);
                    }
                    id if id == IDCANCEL as u32 => {
                        // SAFETY: hwnd is the dialog window.
                        unsafe {
                            DestroyWindow(hwnd);
                        }
                    }
                    _ => {}
                }
            }
            0
        }
        WM_CLOSE => {
            // SAFETY: hwnd is the dialog window.
            unsafe {
                DestroyWindow(hwnd);
            }
            0
        }
        WM_NCDESTROY => {
            // SAFETY: Clear the non-owning context pointer before the window handle dies.
            unsafe {
                SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0);
                DefWindowProcW(hwnd, message, wparam, lparam)
            }
        }
        _ => {
            // SAFETY: All messages not handled by this window procedure are delegated to Windows.
            unsafe { DefWindowProcW(hwnd, message, wparam, lparam) }
        }
    }
}

unsafe extern "system" fn language_config_dialog_proc(
    hwnd: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match message {
        WM_NCCREATE => {
            // SAFETY: For WM_NCCREATE, lparam is a CREATESTRUCTW pointer supplied by Windows.
            let create = unsafe { &*(lparam as *const CREATESTRUCTW) };
            let context = create.lpCreateParams as *mut LanguageConfigDialogContext;
            // SAFETY: The context pointer is supplied by show_language_config_dialog and remains
            // valid until WM_NCDESTROY clears GWLP_USERDATA. DefWindowProcW keeps default
            // non-client initialization, including applying the window caption.
            unsafe {
                SetWindowLongPtrW(hwnd, GWLP_USERDATA, context as isize);
                DefWindowProcW(hwnd, message, wparam, lparam)
            }
        }
        WM_COMMAND => {
            // SAFETY: The userdata pointer is set during WM_NCCREATE and cleared at WM_NCDESTROY.
            if let Some(context) = unsafe { language_config_dialog_context_mut(hwnd) } {
                match low_word(wparam) {
                    id if id == CONTROL_LANGUAGE_CONFIG_DEFAULT_BUTTON_ID as u32 => {
                        language_config_dialog_reset_default(context);
                    }
                    id if id == IDOK as u32 => {
                        language_config_dialog_try_accept(hwnd, context);
                    }
                    id if id == IDCANCEL as u32 => {
                        // SAFETY: hwnd is the dialog window.
                        unsafe {
                            DestroyWindow(hwnd);
                        }
                    }
                    _ => {}
                }
            }
            0
        }
        WM_CLOSE => {
            // SAFETY: hwnd is the dialog window.
            unsafe {
                DestroyWindow(hwnd);
            }
            0
        }
        WM_NCDESTROY => {
            // SAFETY: Clear the non-owning context pointer before the window handle dies.
            unsafe {
                SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0);
                DefWindowProcW(hwnd, message, wparam, lparam)
            }
        }
        _ => {
            // SAFETY: All messages not handled by this window procedure are delegated to Windows.
            unsafe { DefWindowProcW(hwnd, message, wparam, lparam) }
        }
    }
}

unsafe extern "system" fn about_dialog_proc(
    hwnd: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match message {
        WM_NCCREATE => {
            // SAFETY: For WM_NCCREATE, lparam is a CREATESTRUCTW pointer supplied by Windows.
            let create = unsafe { &*(lparam as *const CREATESTRUCTW) };
            let context = create.lpCreateParams as *mut AboutDialogContext;
            // SAFETY: The context pointer is supplied by show_about_dialog and remains valid until
            // WM_NCDESTROY clears GWLP_USERDATA. DefWindowProcW keeps default non-client setup.
            unsafe {
                SetWindowLongPtrW(hwnd, GWLP_USERDATA, context as isize);
                DefWindowProcW(hwnd, message, wparam, lparam)
            }
        }
        WM_COMMAND => {
            match low_word(wparam) {
                id if id == IDOK as u32 || id == IDCANCEL as u32 => {
                    // SAFETY: hwnd is the dialog window.
                    unsafe {
                        DestroyWindow(hwnd);
                    }
                }
                _ => {}
            }
            0
        }
        WM_NOTIFY => {
            if about_dialog_handle_notify(hwnd, lparam) {
                return 0;
            }

            // SAFETY: Unhandled notifications are delegated to Windows.
            unsafe { DefWindowProcW(hwnd, message, wparam, lparam) }
        }
        WM_CLOSE => {
            // SAFETY: hwnd is the dialog window.
            unsafe {
                DestroyWindow(hwnd);
            }
            0
        }
        WM_NCDESTROY => {
            // SAFETY: Clear the non-owning context pointer before the window handle dies.
            unsafe {
                SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0);
                DefWindowProcW(hwnd, message, wparam, lparam)
            }
        }
        _ => {
            // SAFETY: All messages not handled by this window procedure are delegated to Windows.
            unsafe { DefWindowProcW(hwnd, message, wparam, lparam) }
        }
    }
}

fn about_dialog_handle_notify(hwnd: HWND, lparam: LPARAM) -> bool {
    if lparam == 0 {
        return false;
    }

    // SAFETY: For WM_NOTIFY, lparam points to an NMHDR-compatible notification structure.
    let notification = unsafe { &*(lparam as *const NMHDR) };
    if notification.idFrom != CONTROL_ABOUT_LINK_ID as usize {
        return false;
    }
    if notification.code != NM_CLICK && notification.code != NM_RETURN {
        return false;
    }

    // SAFETY: The userdata pointer is set during WM_NCCREATE and cleared at WM_NCDESTROY.
    let language = unsafe { about_dialog_context_mut(hwnd) }
        .map(|context| context.language)
        .unwrap_or(UiLanguage::English);
    open_about_repository_link(hwnd, language);
    true
}

unsafe extern "system" fn text_input_dialog_proc(
    hwnd: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match message {
        WM_NCCREATE => {
            // SAFETY: For WM_NCCREATE, lparam is a CREATESTRUCTW pointer supplied by Windows.
            let create = unsafe { &*(lparam as *const CREATESTRUCTW) };
            let context = create.lpCreateParams as *mut TextInputDialogContext;
            // SAFETY: The context pointer is supplied by show_text_input_dialog and remains valid
            // until WM_NCDESTROY clears GWLP_USERDATA. DefWindowProcW keeps default non-client
            // initialization, including applying the window caption supplied to CreateWindowExW.
            unsafe {
                SetWindowLongPtrW(hwnd, GWLP_USERDATA, context as isize);
                DefWindowProcW(hwnd, message, wparam, lparam)
            }
        }
        WM_COMMAND => {
            // SAFETY: The userdata pointer is set during WM_NCCREATE and cleared at WM_NCDESTROY.
            if let Some(context) = unsafe { text_input_dialog_context_mut(hwnd) } {
                match low_word(wparam) {
                    id if id == IDOK as u32 => {
                        text_input_dialog_try_accept(hwnd, context);
                    }
                    id if id == IDCANCEL as u32 => {
                        // SAFETY: hwnd is the dialog window.
                        unsafe {
                            DestroyWindow(hwnd);
                        }
                    }
                    _ => {}
                }
            }
            0
        }
        WM_CLOSE => {
            // SAFETY: hwnd is the dialog window.
            unsafe {
                DestroyWindow(hwnd);
            }
            0
        }
        WM_NCDESTROY => {
            // SAFETY: Clear the non-owning context pointer before the window handle dies.
            unsafe {
                SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0);
                DefWindowProcW(hwnd, message, wparam, lparam)
            }
        }
        _ => {
            // SAFETY: All messages not handled by this window procedure are delegated to Windows.
            unsafe { DefWindowProcW(hwnd, message, wparam, lparam) }
        }
    }
}

unsafe fn workspace_dialog_context_mut(hwnd: HWND) -> Option<&'static mut WorkspaceDialogContext> {
    // SAFETY: The value was stored by this module as a WorkspaceDialogContext pointer.
    let pointer = unsafe { GetWindowLongPtrW(hwnd, GWLP_USERDATA) } as *mut WorkspaceDialogContext;
    if pointer.is_null() {
        None
    } else {
        // SAFETY: The pointer is non-owning and valid for the current message dispatch.
        unsafe { pointer.as_mut() }
    }
}

unsafe fn command_button_dialog_context_mut(
    hwnd: HWND,
) -> Option<&'static mut CommandButtonDialogContext> {
    // SAFETY: The value was stored by this module as a CommandButtonDialogContext pointer.
    let pointer =
        unsafe { GetWindowLongPtrW(hwnd, GWLP_USERDATA) } as *mut CommandButtonDialogContext;
    if pointer.is_null() {
        None
    } else {
        // SAFETY: The pointer is non-owning and valid for the current message dispatch.
        unsafe { pointer.as_mut() }
    }
}

unsafe fn font_dialog_context_mut(hwnd: HWND) -> Option<&'static mut FontDialogContext> {
    // SAFETY: The value was stored by this module as a FontDialogContext pointer.
    let pointer = unsafe { GetWindowLongPtrW(hwnd, GWLP_USERDATA) } as *mut FontDialogContext;
    if pointer.is_null() {
        None
    } else {
        // SAFETY: The pointer is non-owning and valid for the current message dispatch.
        unsafe { pointer.as_mut() }
    }
}

unsafe fn language_config_dialog_context_mut(
    hwnd: HWND,
) -> Option<&'static mut LanguageConfigDialogContext> {
    // SAFETY: The value was stored by this module as a LanguageConfigDialogContext pointer.
    let pointer =
        unsafe { GetWindowLongPtrW(hwnd, GWLP_USERDATA) } as *mut LanguageConfigDialogContext;
    if pointer.is_null() {
        None
    } else {
        // SAFETY: The pointer is non-owning and valid for the current message dispatch.
        unsafe { pointer.as_mut() }
    }
}

unsafe fn about_dialog_context_mut(hwnd: HWND) -> Option<&'static mut AboutDialogContext> {
    // SAFETY: The value was stored by this module as an AboutDialogContext pointer.
    let pointer = unsafe { GetWindowLongPtrW(hwnd, GWLP_USERDATA) } as *mut AboutDialogContext;
    if pointer.is_null() {
        None
    } else {
        // SAFETY: The pointer is non-owning and valid for the current message dispatch.
        unsafe { pointer.as_mut() }
    }
}

unsafe fn text_input_dialog_context_mut(hwnd: HWND) -> Option<&'static mut TextInputDialogContext> {
    // SAFETY: The value was stored by this module as a TextInputDialogContext pointer.
    let pointer = unsafe { GetWindowLongPtrW(hwnd, GWLP_USERDATA) } as *mut TextInputDialogContext;
    if pointer.is_null() {
        None
    } else {
        // SAFETY: The pointer is non-owning and valid for the current message dispatch.
        unsafe { pointer.as_mut() }
    }
}
