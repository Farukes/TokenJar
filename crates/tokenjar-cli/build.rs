fn main() {
    #[cfg(windows)]
    {
        let mut res = winres::WindowsResource::new();
        res.set_language(0x0409); // English (US)
        res.set_version_info(winres::VersionInfo::FILEVERSION, 0x0001000000030000);
        res.set_version_info(winres::VersionInfo::PRODUCTVERSION, 0x0001000000030000);
        res.set(
            "FileDescription",
            "TokenJar - AI Context and Token Optimization Engine",
        );
        res.set("ProductName", "TokenJar");
        res.set("LegalCopyright", "Copyright (C) 2026 Ömer Faruk Eskitürk");
        res.set("OriginalFilename", "tokenjar.exe");
        res.set("InternalName", "tokenjar");
        res.set("CompanyName", "TokenJar Open Source");
        res.set("FileVersion", "1.0.3.0");
        res.set("ProductVersion", "1.0.3.0");
        res.set_icon("assets/icon.ico");
        res.set_manifest(
            r#"
<assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0">
<trustInfo xmlns="urn:schemas-microsoft-com:asm.v3">
    <security>
        <requestedPrivileges>
            <requestedExecutionLevel level="asInvoker" uiAccess="false" />
        </requestedPrivileges>
    </security>
</trustInfo>
<compatibility xmlns="urn:schemas-microsoft-com:compatibility.v1">
    <application>
        <!-- Windows 10 and Windows 11 -->
        <supportedOS Id="{8e0f7a12-bfb3-4fe8-b9a5-48fd50a15a9a}"/>
        <!-- Windows 8.1 -->
        <supportedOS Id="{1f676c76-80e1-4239-95bb-83d0f6d0da78}"/>
        <!-- Windows 8 -->
        <supportedOS Id="{4a2f28e3-53b9-4441-ba9c-d69d4a4a6e38}"/>
        <!-- Windows 7 -->
        <supportedOS Id="{35138b9a-5d96-4fbd-8e2d-a2440225f93a}"/>
    </application>
</compatibility>
</assembly>
"#,
        );
        if let Err(e) = res.compile() {
            eprintln!("cargo:warning=winres compilation failed: {}", e);
        }
    }
}
