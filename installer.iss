[Setup]
AppName=AI Token & Cost Estimator
AppVersion={#AppVersion}
DefaultDirName={autopf}\AI Token & Cost Estimator
DefaultGroupName=AI Token & Cost Estimator
UninstallDisplayIcon={app}\ai-token-cost-estimator.exe
Compression=lzma2
SolidCompression=yes
OutputDir=staging
OutputBaseFilename=ai-token-cost-estimator-setup-x86_64

[Files]
Source: "target\release\ai-token-cost-estimator.exe"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{group}\AI Token & Cost Estimator"; Filename: "{app}\ai-token-cost-estimator.exe"
Name: "{autodesktop}\AI Token & Cost Estimator"; Filename: "{app}\ai-token-cost-estimator.exe"
