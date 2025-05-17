use steamworks_sys::{AppId_t, CSteamID};
use sys::InputHandle_t;

use super::*;

/// Access to the steam input interface
pub struct Input {
    pub(crate) input: *mut sys::ISteamInput,
    pub(crate) _inner: Arc<Inner>,
}

#[derive(Copy, Clone, Debug)]
pub enum InputType {
    Unknown,
    SteamController,
    XBox360Controller,
    XBoxOneController,
    GenericGamepad,
    PS4Controller,
    AppleMFiController,
    AndroidController,
    SwitchJoyConPair,
    SwitchJoyConSingle,
    SwitchProController,
    MobileTouch,
    PS3Controller,
    PS5Controller,
    SteamDeckController,
}

impl Input {
    /// Init must be called when starting use of this interface.
    /// if explicitly_call_run_frame is called then you will need to manually call RunFrame
    /// each frame, otherwise Steam Input will updated when SteamAPI_RunCallbacks() is called
    pub fn init(&self, explicitly_call_run_frame: bool) -> bool {
        unsafe { sys::SteamAPI_ISteamInput_Init(self.input, explicitly_call_run_frame) }
    }

    /// Synchronize API state with the latest Steam Input action data available. This
    /// is performed automatically by SteamAPI_RunCallbacks, but for the absolute lowest
    /// possible latency, you call this directly before reading controller state.
    /// Note: This must be called from somewhere before GetConnectedControllers will
    /// return any handles
    pub fn run_frame(&self) {
        unsafe { sys::SteamAPI_ISteamInput_RunFrame(self.input, false) }
    }

    // Enable SteamInputDeviceConnected_t and SteamInputDeviceDisconnected_t callbacks.
    //
    // Each controller that is already connected will generate a device connected
    // callback when you enable them
    pub fn enable_device_callbacks(&self) {
        unsafe { sys::SteamAPI_ISteamInput_EnableDeviceCallbacks(self.input) }
    }

    /// Returns a list of the currently connected controllers
    pub fn get_connected_controllers(&self) -> Vec<sys::InputHandle_t> {
        let mut handles = vec![0_u64; sys::STEAM_INPUT_MAX_COUNT as usize];
        let quantity = self.get_connected_controllers_slice(&mut handles);
        handles.shrink_to(quantity);
        handles
    }

    /// Returns a list of the currently connected controllers without allocating, and the count
    pub fn get_connected_controllers_slice(
        &self,
        mut controllers: impl AsMut<[InputHandle_t]>,
    ) -> usize {
        let handles = controllers.as_mut();
        assert!(handles.len() >= sys::STEAM_INPUT_MAX_COUNT as usize);
        unsafe {
            return sys::SteamAPI_ISteamInput_GetConnectedControllers(
                self.input,
                handles.as_mut_ptr(),
            ) as usize;
        }
    }

    /// Returns the associated controller handle for the specified emulated gamepad
    ///
    /// Returns the associated controller handle for the specified emulated gamepad.
    /// Can be used with GetInputTypeForHandle to determine the type of controller using Steam Input Gamepad Emulation.
    pub fn get_controller_for_gamepad_index(&self, index: i32) -> sys::InputHandle_t {
        unsafe { sys::SteamAPI_ISteamInput_GetControllerForGamepadIndex(self.input, index) }
    }

    /// Allows to load a specific Action Manifest File localy
    pub fn set_input_action_manifest_file_path(&self, path: &str) -> bool {
        let path = CString::new(path).unwrap();
        unsafe {
            sys::SteamAPI_ISteamInput_SetInputActionManifestFilePath(self.input, path.as_ptr())
        }
    }

    /// Returns the associated ControllerActionSet handle for the specified controller,
    pub fn get_action_set_handle(&self, action_set_name: &str) -> sys::InputActionSetHandle_t {
        let name = CString::new(action_set_name).unwrap();
        unsafe { sys::SteamAPI_ISteamInput_GetActionSetHandle(self.input, name.as_ptr()) }
    }

    /// Returns the input type for a controler
    pub fn get_input_type_for_handle(&self, input_handle: sys::InputHandle_t) -> InputType {
        let input_type: sys::ESteamInputType =
            unsafe { sys::SteamAPI_ISteamInput_GetInputTypeForHandle(self.input, input_handle) };

        match input_type {
            sys::ESteamInputType::k_ESteamInputType_SteamController => InputType::SteamController,
            sys::ESteamInputType::k_ESteamInputType_GenericGamepad => InputType::GenericGamepad,
            sys::ESteamInputType::k_ESteamInputType_PS4Controller => InputType::PS4Controller,
            sys::ESteamInputType::k_ESteamInputType_SwitchJoyConPair => InputType::SwitchJoyConPair,
            sys::ESteamInputType::k_ESteamInputType_MobileTouch => InputType::MobileTouch,
            sys::ESteamInputType::k_ESteamInputType_PS3Controller => InputType::PS3Controller,
            sys::ESteamInputType::k_ESteamInputType_PS5Controller => InputType::PS5Controller,
            sys::ESteamInputType::k_ESteamInputType_XBox360Controller => {
                InputType::XBox360Controller
            }
            sys::ESteamInputType::k_ESteamInputType_XBoxOneController => {
                InputType::XBoxOneController
            }
            sys::ESteamInputType::k_ESteamInputType_AppleMFiController => {
                InputType::AppleMFiController
            }
            sys::ESteamInputType::k_ESteamInputType_AndroidController => {
                InputType::AndroidController
            }
            sys::ESteamInputType::k_ESteamInputType_SwitchJoyConSingle => {
                InputType::SwitchJoyConSingle
            }
            sys::ESteamInputType::k_ESteamInputType_SwitchProController => {
                InputType::SwitchProController
            }
            sys::ESteamInputType::k_ESteamInputType_SteamDeckController => {
                InputType::SteamDeckController
            }
            _ => InputType::Unknown,
        }
    }

    /// Returns the glyph for an input action
    pub fn get_glyph_for_action_origin(&self, action_origin: sys::EInputActionOrigin) -> String {
        unsafe {
            let glyph_path =
                sys::SteamAPI_ISteamInput_GetGlyphForActionOrigin_Legacy(self.input, action_origin);
            let glyph_path = CStr::from_ptr(glyph_path);
            glyph_path.to_string_lossy().into_owned()
        }
    }

    /// Returns the name of an input action
    pub fn get_string_for_action_origin(&self, action_origin: sys::EInputActionOrigin) -> String {
        unsafe {
            let name_path =
                sys::SteamAPI_ISteamInput_GetStringForActionOrigin(self.input, action_origin);
            let name_path = CStr::from_ptr(name_path);
            name_path.to_string_lossy().into_owned()
        }
    }

    /// Reconfigure the controller to use the specified action set
    /// This is cheap, and can be safely called repeatedly.
    pub fn activate_action_set_handle(
        &self,
        input_handle: sys::InputHandle_t,
        action_set_handle: sys::InputActionSetHandle_t,
    ) {
        unsafe {
            sys::SteamAPI_ISteamInput_ActivateActionSet(self.input, input_handle, action_set_handle)
        }
    }

    /// Reconfigure the controller to use the specified action set layer
    pub fn activate_action_set_layer_handle(
        &self,
        input_handle: sys::InputHandle_t,
        action_set_handle: sys::InputActionSetHandle_t,
    ) {
        unsafe {
            sys::SteamAPI_ISteamInput_ActivateActionSetLayer(
                self.input,
                input_handle,
                action_set_handle,
            )
        }
    }

    /// Reconfigure the controller to stop using the specified action set layer
    pub fn deactivate_action_set_layer_handle(
        &self,
        input_handle: sys::InputHandle_t,
        action_set_handle: sys::InputActionSetHandle_t,
    ) {
        unsafe {
            sys::SteamAPI_ISteamInput_DeactivateActionSetLayer(
                self.input,
                input_handle,
                action_set_handle,
            )
        }
    }

    /// Reconfigure the controller to stop using the specified action set layer
    pub fn deactivate_all_action_set_layers(&self, input_handle: sys::InputHandle_t) {
        unsafe { sys::SteamAPI_ISteamInput_DeactivateAllActionSetLayers(self.input, input_handle) }
    }

    /// Get the handle of the specified Digital action.
    pub fn get_digital_action_handle(&self, action_name: &str) -> sys::InputDigitalActionHandle_t {
        let name = CString::new(action_name).unwrap();
        unsafe { sys::SteamAPI_ISteamInput_GetDigitalActionHandle(self.input, name.as_ptr()) }
    }

    /// Get the handle of the specified Analog action.
    pub fn get_analog_action_handle(&self, action_name: &str) -> sys::InputAnalogActionHandle_t {
        let name = CString::new(action_name).unwrap();
        unsafe { sys::SteamAPI_ISteamInput_GetAnalogActionHandle(self.input, name.as_ptr()) }
    }

    /// Returns the current state of the supplied digital game action.
    pub fn get_digital_action_data(
        &self,
        input_handle: sys::InputHandle_t,
        action_handle: sys::InputDigitalActionHandle_t,
    ) -> sys::InputDigitalActionData_t {
        unsafe {
            sys::SteamAPI_ISteamInput_GetDigitalActionData(self.input, input_handle, action_handle)
        }
    }

    /// Returns the current state of the supplied analog game action.
    pub fn get_analog_action_data(
        &self,
        input_handle: sys::InputHandle_t,
        action_handle: sys::InputAnalogActionHandle_t,
    ) -> sys::InputAnalogActionData_t {
        unsafe {
            sys::SteamAPI_ISteamInput_GetAnalogActionData(self.input, input_handle, action_handle)
        }
    }

    /// Get the origin(s) for a digital action within an action set.
    pub fn get_digital_action_origins(
        &self,
        input_handle: sys::InputHandle_t,
        action_set_handle: sys::InputActionSetHandle_t,
        digital_action_handle: sys::InputDigitalActionHandle_t,
    ) -> Vec<sys::EInputActionOrigin> {
        unsafe {
            let mut origins = Vec::with_capacity(sys::STEAM_INPUT_MAX_ORIGINS as usize);
            let len = sys::SteamAPI_ISteamInput_GetDigitalActionOrigins(
                self.input,
                input_handle,
                action_set_handle,
                digital_action_handle,
                origins.as_mut_ptr(),
            );
            origins.set_len(len as usize);
            origins
        }
    }

    /// Get the origin(s) for an analog action within an action set.
    pub fn get_analog_action_origins(
        &self,
        input_handle: sys::InputHandle_t,
        action_set_handle: sys::InputActionSetHandle_t,
        analog_action_handle: sys::InputAnalogActionHandle_t,
    ) -> Vec<sys::EInputActionOrigin> {
        unsafe {
            let mut origins = Vec::with_capacity(sys::STEAM_INPUT_MAX_ORIGINS as usize);
            let len = sys::SteamAPI_ISteamInput_GetAnalogActionOrigins(
                self.input,
                input_handle,
                action_set_handle,
                analog_action_handle,
                origins.as_mut_ptr(),
            );
            origins.set_len(len as usize);
            origins
        }
    }

    pub fn get_motion_data(&self, input_handle: sys::InputHandle_t) -> sys::InputMotionData_t {
        unsafe { sys::SteamAPI_ISteamInput_GetMotionData(self.input, input_handle) }
    }

    /// Invokes the Steam overlay and brings up the binding screen.
    /// Returns true for success, false if overlay is disabled/unavailable.
    /// If the player is using Big Picture Mode the configuration will open in
    /// the overlay. In desktop mode a popup window version of Big Picture will
    /// be created and open the configuration.
    pub fn show_binding_panel(&self, input_handle: sys::InputHandle_t) -> bool {
        unsafe { sys::SteamAPI_ISteamInput_ShowBindingPanel(self.input, input_handle) }
    }

    /// Shutdown must be called when ending use of this interface.
    pub fn shutdown(&self) {
        unsafe {
            sys::SteamAPI_ISteamInput_Shutdown(self.input);
        }
    }
}

const CALLBACK_BASE_ID: i32 = 2800;

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct DeviceConnected {
    pub handle: InputHandle_t,
}

unsafe impl Callback for DeviceConnected {
    const ID: i32 = CALLBACK_BASE_ID + 1;

    unsafe fn from_raw(raw: *mut c_void) -> Self {
        let val = &mut *(raw as *mut sys::SteamInputDeviceConnected_t);
        Self {
            handle: val.m_ulConnectedDeviceHandle,
        }
    }
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct DeviceDisconnected {
    pub handle: InputHandle_t,
}

unsafe impl Callback for DeviceDisconnected {
    const ID: i32 = CALLBACK_BASE_ID + 2;

    unsafe fn from_raw(raw: *mut c_void) -> Self {
        let val = &mut *(raw as *mut sys::SteamInputDeviceDisconnected_t);
        Self {
            handle: val.m_ulDisconnectedDeviceHandle,
        }
    }
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ConfigurationLoaded {
    pub app_id: AppId_t,
    pub handle: InputHandle_t,
    // pub m_ulMappingCreator: CSteamID,
    pub major_revision: u32,
    pub minor_revision: u32,
    pub uses_steam_input_api: bool,
    pub uses_gamepad_api: bool,
}

unsafe impl Callback for ConfigurationLoaded {
    const ID: i32 = CALLBACK_BASE_ID + 3;

    unsafe fn from_raw(raw: *mut c_void) -> Self {
        let sys::SteamInputConfigurationLoaded_t {
            m_unAppID,
            m_ulDeviceHandle,
            m_ulMappingCreator: _,
            m_unMajorRevision,
            m_unMinorRevision,
            m_bUsesSteamInputAPI,
            m_bUsesGamepadAPI,
        } = *(raw as *mut sys::SteamInputConfigurationLoaded_t);
        Self {
            app_id: m_unAppID,
            handle: m_ulDeviceHandle,
            major_revision: m_unMajorRevision,
            minor_revision: m_unMinorRevision,
            uses_steam_input_api: m_bUsesSteamInputAPI,
            uses_gamepad_api: m_bUsesGamepadAPI,
        }
    }
}
