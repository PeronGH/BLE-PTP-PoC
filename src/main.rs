mod ble_hid;
mod esp_hid_ffi;
mod feature_reports;
mod hid_descriptor;

use std::time::Duration;

use esp_idf_svc::hal::peripherals::Peripherals;
use esp_idf_svc::nvs::EspDefaultNvsPartition;
use esp_idf_svc::sys::*;
use log::{error, info};

fn main() {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    if let Err(e) = run() {
        error!("Fatal: {e}");
        loop {
            std::thread::sleep(Duration::from_secs(1));
        }
    }
}

fn run() -> Result<(), EspError> {
    info!("BLE PTP PoC starting");

    // NVS is required by the Bluetooth stack.
    let _nvs = EspDefaultNvsPartition::take()?;

    // Peripheral access — needed to prove we own the hardware.
    let _peripherals = Peripherals::take()?;

    // Initialise the Bluetooth controller and Bluedroid stack.
    bt_init()?;

    // Set up GAP advertising + create the HOGP HID device.
    ble_hid::init()?;

    info!("Waiting for BLE connection…");

    // After a host connects, send a hardcoded diagonal touch to move the cursor.
    let mut x: u16 = 5000;
    loop {
        std::thread::sleep(Duration::from_millis(50));

        if !ble_hid::is_connected() {
            continue;
        }

        // Finger down — sweep X from 5000 to 15000.
        if x <= 15000 {
            if let Err(e) = ble_hid::send_touch_report(x, 6000, 1, true) {
                error!("send_touch_report: {e}");
            }
            x += 200;
        } else {
            // Lift the finger, then pause before repeating.
            let _ = ble_hid::send_touch_report(0, 0, 1, false);
            std::thread::sleep(Duration::from_secs(2));
            x = 5000;
        }
    }
}

/// Initialise the Bluetooth controller (BLE-only) and Bluedroid stack.
fn bt_init() -> Result<(), EspError> {
    unsafe {
        // Free Classic-BT memory — we only use BLE.
        esp!(esp_bt_controller_mem_release(
            esp_bt_mode_t_ESP_BT_MODE_CLASSIC_BT
        ))?;

        let mut cfg = bt_controller_default_config();
        esp!(esp_bt_controller_init(&mut cfg))?;
        esp!(esp_bt_controller_enable(esp_bt_mode_t_ESP_BT_MODE_BLE))?;

        esp!(esp_bluedroid_init())?;
        esp!(esp_bluedroid_enable())?;
    }

    info!("Bluetooth stack initialised (BLE)");
    Ok(())
}

/// Reproduce `BT_CONTROLLER_INIT_CONFIG_DEFAULT()` from the ESP-IDF C macro.
///
/// The bindgen `Default` impl zeroes the struct, which is invalid (the
/// controller checks `magic`, `version`, stack size, and priority).
fn bt_controller_default_config() -> esp_bt_controller_config_t {
    esp_bt_controller_config_t {
        magic: ESP_BT_CTRL_CONFIG_MAGIC_VAL,
        version: ESP_BT_CTRL_CONFIG_VERSION,
        controller_task_stack_size: ESP_TASK_BT_CONTROLLER_STACK as _,
        controller_task_prio: ESP_TASK_BT_CONTROLLER_PRIO as _,
        controller_task_run_cpu: CONFIG_BT_CTRL_PINNED_TO_CORE as _,
        bluetooth_mode: CONFIG_BT_CTRL_MODE_EFF as _,
        ble_max_act: CONFIG_BT_CTRL_BLE_MAX_ACT_EFF as _,
        sleep_mode: CONFIG_BT_CTRL_SLEEP_MODE_EFF as _,
        sleep_clock: CONFIG_BT_CTRL_SLEEP_CLOCK_EFF as _,
        ble_st_acl_tx_buf_nb: CONFIG_BT_CTRL_BLE_STATIC_ACL_TX_BUF_NB as _,
        ble_hw_cca_check: CONFIG_BT_CTRL_HW_CCA_EFF as _,
        ble_adv_dup_filt_max: CONFIG_BT_CTRL_ADV_DUP_FILT_MAX as _,
        coex_param_en: false,
        ce_len_type: CONFIG_BT_CTRL_CE_LENGTH_TYPE_EFF as _,
        coex_use_hooks: false,
        hci_tl_type: CONFIG_BT_CTRL_HCI_TL_EFF as _,
        hci_tl_funcs: std::ptr::null_mut(),
        txant_dft: CONFIG_BT_CTRL_TX_ANTENNA_INDEX_EFF as _,
        rxant_dft: CONFIG_BT_CTRL_RX_ANTENNA_INDEX_EFF as _,
        txpwr_dft: CONFIG_BT_CTRL_DFT_TX_POWER_LEVEL_EFF as _,
        cfg_mask: CFG_MASK,
        scan_duplicate_mode: SCAN_DUPLICATE_MODE as _,
        scan_duplicate_type: SCAN_DUPLICATE_TYPE_VALUE as _,
        normal_adv_size: NORMAL_SCAN_DUPLICATE_CACHE_SIZE as _,
        mesh_adv_size: MESH_DUPLICATE_SCAN_CACHE_SIZE as _,
        coex_phy_coded_tx_rx_time_limit: 0,
        hw_target_code: BLE_HW_TARGET_CODE_CHIP_ECO0,
        slave_ce_len_min: SLAVE_CE_LEN_MIN_DEFAULT as _,
        hw_recorrect_en: AGC_RECORRECT_EN as _,
        cca_thresh: CONFIG_BT_CTRL_HW_CCA_VAL as _,
        scan_backoff_upperlimitmax: BT_CTRL_SCAN_BACKOFF_UPPERLIMITMAX as _,
        dup_list_refresh_period: DUPL_SCAN_CACHE_REFRESH_PERIOD as _,
        ble_50_feat_supp: BT_CTRL_50_FEATURE_SUPPORT != 0,
        ble_cca_mode: BT_BLE_CCA_MODE as _,
        ble_data_lenth_zero_aux: BT_BLE_ADV_DATA_LENGTH_ZERO_AUX as _,
        ble_chan_ass_en: BT_CTRL_CHAN_ASS_EN as _,
        ble_ping_en: BT_CTRL_LE_PING_EN as _,
        ble_llcp_disc_flag: BT_CTRL_BLE_LLCP_DISC_FLAG as _,
        run_in_flash: BT_CTRL_RUN_IN_FLASH_ONLY != 0,
        dtm_en: BT_CTRL_DTM_ENABLE != 0,
        enc_en: BLE_SECURITY_ENABLE != 0,
        qa_test: BT_CTRL_BLE_TEST != 0,
        connect_en: BT_CTRL_BLE_MASTER != 0,
        scan_en: BT_CTRL_BLE_SCAN != 0,
        ble_aa_check: BLE_CTRL_CHECK_CONNECT_IND_ACCESS_ADDRESS_ENABLED != 0,
        adv_en: true,
    }
}
