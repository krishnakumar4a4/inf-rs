use inf_rs::{InfEntry, InfValue, WinInfFile};
use std::path::PathBuf;

#[test]
fn test_audiocodec_inf_parsing() {
    let mut inf_file = WinInfFile::default();
    let inf_path = PathBuf::from("tests/fixtures/AudioCodec.inf");
    
    // Test basic parsing
    assert!(inf_file.parse(inf_path).is_ok());
    
    // Test Version section
    let version_section = inf_file.sections.get("Version").unwrap();
    assert_eq!(version_section.entries.len(), 7);
    let (key, value) = read_key_value(version_section.entries[0].clone()).unwrap();
    assert_eq!(key, "Signature");
    assert_eq!(value.unwrap(), InfValue::Raw("$WINDOWS NT$".to_string()));
    
    let (key, value) = read_key_value(version_section.entries[1].clone()).unwrap();
    assert_eq!(key, "Class");
    assert_eq!(value.unwrap(), InfValue::Raw("MEDIA".to_string()));
    
    let (key, value) = read_key_value(version_section.entries[2].clone()).unwrap();
    assert_eq!(key, "ClassGuid");
    assert_eq!(value.unwrap(), InfValue::Raw("{4d36e96c-e325-11ce-bfc1-08002be10318}".to_string()));
    
    let (key, value) = read_key_value(version_section.entries[3].clone()).unwrap();
    assert_eq!(key, "Provider");
    assert_eq!(value.unwrap(), InfValue::Raw("%ProviderName%".to_string()));
    
    let (key, value) = read_key_value(version_section.entries[4].clone()).unwrap();
    assert_eq!(key, "DriverVer");
    assert_eq!(value.unwrap(), InfValue::Raw("07/07/2021, 1.0.0.0".to_string()));
    
    let (key, value) = read_key_value(version_section.entries[5].clone()).unwrap();
    assert_eq!(key, "CatalogFile");
    assert_eq!(value.unwrap(), InfValue::Raw("AudioCodec.cat".to_string()));
    
    let (key, value) = read_key_value(version_section.entries[6].clone()).unwrap();
    assert_eq!(key, "PnpLockDown");
    assert_eq!(value.unwrap(), InfValue::Raw("1".to_string()));

    // Test DestinationDirs section
    let dest_dirs_section = inf_file.sections.get("DestinationDirs").unwrap();
    let (key, value) = read_key_value(dest_dirs_section.entries[0].clone()).unwrap();
    assert_eq!(key, "DefaultDestDir");
    assert_eq!(value.unwrap(), InfValue::Raw("13".to_string()));

    // Test Manufacturer section
    let manufacturer_section = inf_file.sections.get("Manufacturer").unwrap();
    let (key, value) = read_key_value(manufacturer_section.entries[0].clone()).unwrap();
    assert_eq!(key, "%StdMfg%");
    assert_eq!(value.unwrap(), InfValue::Raw("Standard,NT$ARCH$.10.0...19041".to_string()));

    // Test Standard.NT$ARCH$.10.0...19041 section
    let standard_section = inf_file.sections.get("Standard.NT$ARCH$.10.0...19041").unwrap();
    let (key, value) = read_key_value(standard_section.entries[0].clone()).unwrap();
    assert_eq!(key, "%AudioCodec.DeviceDesc%");
    assert_eq!(value.unwrap(), InfValue::Raw("Audio_Device, ROOT\\AudioCodec".to_string()));

    // Test Audio_Device.NT section
    let audio_device_nt_section = inf_file.sections.get("Audio_Device.NT").unwrap();
    let (key, value) = read_key_value(audio_device_nt_section.entries[0].clone()).unwrap();
    assert_eq!(key, "CopyFiles");
    assert_eq!(value.unwrap(), InfValue::Raw("Audio_Device.NT.Copy".to_string()));

    // Test Audio_Device.NT.Copy section
    let audio_device_nt_copy_section = inf_file.sections.get("Audio_Device.NT.Copy").unwrap();
    assert_eq!(audio_device_nt_copy_section.entries.len(), 1);
    let value = read_value_only(audio_device_nt_copy_section.entries[0].clone()).unwrap();
    assert_eq!(value, InfValue::Raw("AudioCodec.sys".to_string()));

    // Test Audio_Device.NT.Services section
    let audio_device_nt_services_section = inf_file.sections.get("Audio_Device.NT.Services").unwrap();
    let (key, value) = read_key_value(audio_device_nt_services_section.entries[0].clone()).unwrap();
    assert_eq!(key, "AddService");
    assert_eq!(value.unwrap(), InfValue::Raw("AudioCodec, %SPSVCINST_ASSOCSERVICE%, Audio_Service_Inst".to_string()));

    // Test Audio_Service_Inst section
    let audio_service_inst_section = inf_file.sections.get("Audio_Service_Inst").unwrap();
    assert_eq!(audio_service_inst_section.entries.len(), 5);
    let (key, value) = read_key_value(audio_service_inst_section.entries[0].clone()).unwrap();
    assert_eq!(key, "DisplayName");
    assert_eq!(value.unwrap(), InfValue::Raw("%AudioCodec.DeviceDesc%".to_string()));
    let (key, value) = read_key_value(audio_service_inst_section.entries[1].clone()).unwrap();
    assert_eq!(key, "ServiceType");
    assert_eq!(value.unwrap(), InfValue::Raw("1".to_string()));
    let (key, value) = read_key_value(audio_service_inst_section.entries[2].clone()).unwrap();
    assert_eq!(key, "StartType");
    assert_eq!(value.unwrap(), InfValue::Raw("3".to_string()));
    let (key, value) = read_key_value(audio_service_inst_section.entries[3].clone()).unwrap();
    assert_eq!(key, "ErrorControl");
    assert_eq!(value.unwrap(), InfValue::Raw("1".to_string()));
    let (key, value) = read_key_value(audio_service_inst_section.entries[4].clone()).unwrap();
    assert_eq!(key, "ServiceBinary");
    assert_eq!(value.unwrap(), InfValue::Raw("%13%\\AudioCodec.sys".to_string()));

    // Test SourceDisksNames section
    let source_disks_names_section = inf_file.sections.get("SourceDisksNames").unwrap();
    let (key, value) = read_key_value(source_disks_names_section.entries[0].clone()).unwrap();
    assert_eq!(key, "1");
    assert_eq!(value.unwrap(), InfValue::Raw("%DiskId1%,,,\"\"".to_string()));

    // Test SourceDisksFiles section
    let source_disks_files_section = inf_file.sections.get("SourceDisksFiles").unwrap();
    let (key, value) = read_key_value(source_disks_files_section.entries[0].clone()).unwrap();
    assert_eq!(key, "AudioCodec.sys");
    assert_eq!(value.unwrap(), InfValue::Raw("1,,".to_string()));

    // Test Audio_Device.NT.Wdf section
    let audio_device_nt_wdf_section = inf_file.sections.get("Audio_Device.NT.Wdf").unwrap();
    let (key, value) = read_key_value(audio_device_nt_wdf_section.entries[0].clone()).unwrap();
    assert_eq!(key, "KmdfService");
    assert_eq!(value.unwrap(), InfValue::Raw("AudioCodec, Audio_wdfsect".to_string()));

    // Test Audio_wdfsect section
    let audio_wdfsect_section = inf_file.sections.get("Audio_wdfsect").unwrap();
    let (key, value) = read_key_value(audio_wdfsect_section.entries[0].clone()).unwrap();
    assert_eq!(key, "KmdfLibraryVersion");
    assert_eq!(value.unwrap(), InfValue::Raw("$KMDFVERSION$".to_string()));
    
    // Test Audio_Device.NT.Interfaces section
    let audio_device_nt_interfaces_section = inf_file.sections.get("Audio_Device.NT.Interfaces").unwrap();
    assert_eq!(audio_device_nt_interfaces_section.entries.len(), 6);
    
    // Test render endpoint interfaces
    let (key, value) = read_key_value(audio_device_nt_interfaces_section.entries[0].clone()).unwrap();
    assert_eq!(key, "AddInterface");
    assert_eq!(value.unwrap(), InfValue::Raw("%KSCATEGORY_AUDIO%,    %KSNAME_Speaker%,  Audio_Device.I.Speaker".to_string()));
    
    let (key, value) = read_key_value(audio_device_nt_interfaces_section.entries[1].clone()).unwrap();
    assert_eq!(key, "AddInterface"); 
    assert_eq!(value.unwrap(), InfValue::Raw("%KSCATEGORY_RENDER%,   %KSNAME_Speaker%,  Audio_Device.I.Speaker".to_string()));
    
    let (key, value) = read_key_value(audio_device_nt_interfaces_section.entries[2].clone()).unwrap();
    assert_eq!(key, "AddInterface");
    assert_eq!(value.unwrap(), InfValue::Raw("%KSCATEGORY_REALTIME%, %KSNAME_Speaker%,  Audio_Device.I.Speaker".to_string()));

    // Test mic capture endpoint interfaces
    let (key, value) = read_key_value(audio_device_nt_interfaces_section.entries[3].clone()).unwrap();
    assert_eq!(key, "AddInterface");
    assert_eq!(value.unwrap(), InfValue::Raw("%KSCATEGORY_AUDIO%,    %KSNAME_Microphone%, Audio_Device.I.Microphone".to_string()));
    
    let (key, value) = read_key_value(audio_device_nt_interfaces_section.entries[4].clone()).unwrap();
    assert_eq!(key, "AddInterface");
    assert_eq!(value.unwrap(), InfValue::Raw("%KSCATEGORY_CAPTURE%,  %KSNAME_Microphone%, Audio_Device.I.Microphone".to_string()));
    
    let (key, value) = read_key_value(audio_device_nt_interfaces_section.entries[5].clone()).unwrap();
    assert_eq!(key, "AddInterface");
    assert_eq!(value.unwrap(), InfValue::Raw("%KSCATEGORY_REALTIME%, %KSNAME_Microphone%, Audio_Device.I.Microphone".to_string()));

    // Test Strings section
    let strings_section = inf_file.sections.get("Strings").unwrap();
    assert!(strings_section.entries.len() >= 4);
    
    let (key, value) = read_key_value(strings_section.entries[0].clone()).unwrap();
    assert_eq!(key, "KSNAME_Speaker");
    assert_eq!(value.unwrap(), InfValue::Raw("Speaker0".to_string()));
    
    let (key, value) = read_key_value(strings_section.entries[1].clone()).unwrap();
    assert_eq!(key, "KSNAME_Microphone"); 
    assert_eq!(value.unwrap(), InfValue::Raw("Microphone0".to_string()));
    
    let (key, value) = read_key_value(strings_section.entries[2].clone()).unwrap();
    assert_eq!(key, "SPSVCINST_ASSOCSERVICE");
    assert_eq!(value.unwrap(), InfValue::Raw("0x00000002".to_string()));
    
    let (key, value) = read_key_value(strings_section.entries[3].clone()).unwrap();
    assert_eq!(key, "ProviderName");
    assert_eq!(value.unwrap(), InfValue::Raw("VS_Microsoft".to_string()));
    // Test remaining Strings section entries
    let (key, value) = read_key_value(strings_section.entries[4].clone()).unwrap();
    assert_eq!(key, "Proxy.CLSID");
    assert_eq!(value.unwrap(), InfValue::Raw("{17CCA71B-ECD7-11D0-B908-00A0C9223196}".to_string()));

    let (key, value) = read_key_value(strings_section.entries[5].clone()).unwrap();
    assert_eq!(key, "KSCATEGORY_AUDIO");
    assert_eq!(value.unwrap(), InfValue::Raw("{6994AD04-93EF-11D0-A3CC-00A0C9223196}".to_string()));

    let (key, value) = read_key_value(strings_section.entries[6].clone()).unwrap();
    assert_eq!(key, "KSCATEGORY_RENDER");
    assert_eq!(value.unwrap(), InfValue::Raw("{65E8773E-8F56-11D0-A3B9-00A0C9223196}".to_string()));

    let (key, value) = read_key_value(strings_section.entries[7].clone()).unwrap();
    assert_eq!(key, "KSCATEGORY_CAPTURE");
    assert_eq!(value.unwrap(), InfValue::Raw("{65E8773D-8F56-11D0-A3B9-00A0C9223196}".to_string()));

    let (key, value) = read_key_value(strings_section.entries[8].clone()).unwrap();
    assert_eq!(key, "KSCATEGORY_REALTIME");
    assert_eq!(value.unwrap(), InfValue::Raw("{EB115FFC-10C8-4964-831D-6DCB02E6F23F}".to_string()));

    let (key, value) = read_key_value(strings_section.entries[9].clone()).unwrap();
    assert_eq!(key, "KSNODETYPE_ANY");
    assert_eq!(value.unwrap(), InfValue::Raw("{00000000-0000-0000-0000-000000000000}".to_string()));

    let (key, value) = read_key_value(strings_section.entries[10].clone()).unwrap();
    assert_eq!(key, "PKEY_AudioEndpoint_ControlPanelPageProvider");
    assert_eq!(value.unwrap(), InfValue::Raw("{1DA5D803-D492-4EDD-8C23-E0C0FFEE7F0E},1".to_string()));

    let (key, value) = read_key_value(strings_section.entries[11].clone()).unwrap();
    assert_eq!(key, "PKEY_AudioEndpoint_Association");
    assert_eq!(value.unwrap(), InfValue::Raw("{1DA5D803-D492-4EDD-8C23-E0C0FFEE7F0E},2".to_string()));

    let (key, value) = read_key_value(strings_section.entries[12].clone()).unwrap();
    assert_eq!(key, "PKEY_AudioEndpoint_Supports_EventDriven_Mode");
    assert_eq!(value.unwrap(), InfValue::Raw("{1DA5D803-D492-4EDD-8C23-E0C0FFEE7F0E},7".to_string()));

    let (key, value) = read_key_value(strings_section.entries[13].clone()).unwrap();
    assert_eq!(key, "PKEY_AudioEndpoint_Default_VolumeInDb");
    assert_eq!(value.unwrap(), InfValue::Raw("{1DA5D803-D492-4EDD-8C23-E0C0FFEE7F0E},9".to_string()));

    let (key, value) = read_key_value(strings_section.entries[14].clone()).unwrap();
    assert_eq!(key, "StdMfg");
    assert_eq!(value.unwrap(), InfValue::Raw("AudioCodec Device".to_string()));

    let (key, value) = read_key_value(strings_section.entries[15].clone()).unwrap();
    assert_eq!(key, "DiskId1");
    assert_eq!(value.unwrap(), InfValue::Raw("AudioCodec Installation Disk".to_string()));

    let (key, value) = read_key_value(strings_section.entries[16].clone()).unwrap();
    assert_eq!(key, "AudioCodec.DeviceDesc");
    assert_eq!(value.unwrap(), InfValue::Raw("AudioCodec Device".to_string()));

    let (key, value) = read_key_value(strings_section.entries[17].clone()).unwrap();
    assert_eq!(key, "Audio_Device.Speaker.szPname");
    assert_eq!(value.unwrap(), InfValue::Raw("AudioCodec Speaker".to_string()));

    let (key, value) = read_key_value(strings_section.entries[18].clone()).unwrap();
    assert_eq!(key, "Audio_Device.Microphone.szPname");
    assert_eq!(value.unwrap(), InfValue::Raw("AudioCodec Microphone".to_string()));
}

#[test]
fn test_inf_file_not_found() {
    let mut inf_file = WinInfFile::default();
    let inf_path = PathBuf::from("tests/fixtures/nonexistent.inf");
    
    // Test file not found error
    assert!(inf_file.parse(inf_path).is_err());
}

#[test]
fn test_inf_file_non_existent_section() {
    let mut inf_file = WinInfFile::default();
    let inf_path = PathBuf::from("tests/fixtures/Intel.inf");
    
    // Test invalid section name
    assert!(inf_file.parse(inf_path).is_ok());
    let invalid_section = inf_file.sections.get("[Non Existent Section]");
    assert!(invalid_section.is_none());
}

fn read_key_value(entry: InfEntry) -> Option<(String, Option<InfValue>)> {
    if let InfEntry::KeyValue(key, value) = entry {
        Some((key.clone(), value.clone()))
    } else {
        None
    }
}

fn read_value_only(entry: InfEntry) -> Option<InfValue> {
    if let InfEntry::OnlyValue(value) = entry {
        Some(value)
    } else {
        None
    }
}