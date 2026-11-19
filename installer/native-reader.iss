#define AppName "Native Reader"
#define AppVersion "0.1.2"
#define AppPublisher "Native Reader"

[Setup]
AppId={{79C053D4-A525-4120-B177-F8D1A6F27D3B}
AppName={#AppName}
AppVersion={#AppVersion}
AppVerName={#AppName} {#AppVersion}
AppPublisher={#AppPublisher}
DefaultDirName={localappdata}\Programs\Native Reader
DefaultGroupName={#AppName}
DisableProgramGroupPage=yes
PrivilegesRequired=lowest
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
MinVersion=10.0.15063
OutputDir=..\dist
OutputBaseFilename=NativeReader-Setup-{#AppVersion}
SetupIconFile=..\assets\native-reader.ico
UninstallDisplayIcon={app}\native-reader.exe
Compression=lzma2/ultra64
SolidCompression=yes
WizardStyle=modern
AllowNoIcons=yes
CloseApplications=yes
RestartApplications=no
VersionInfoVersion={#AppVersion}.0
VersionInfoDescription=Native Reader 安装程序
VersionInfoProductName={#AppName}
VersionInfoCompany={#AppPublisher}
VersionInfoCopyright=Native Reader

[Languages]
Name: "chinesesimplified"; MessagesFile: "compiler:Languages\ChineseSimplified.isl"

[Tasks]
Name: "desktopicon"; Description: "创建桌面快捷方式"; GroupDescription: "附加任务："; Flags: unchecked

[Files]
Source: "..\target\release\native-reader.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\assets\native-reader.ico"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\LICENSE"; DestDir: "{app}"; DestName: "LICENSE.txt"; Flags: ignoreversion
Source: "..\THIRD_PARTY_NOTICES.md"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\licenses\*"; DestDir: "{app}\licenses"; Flags: ignoreversion

[Icons]
Name: "{group}\{#AppName}"; Filename: "{app}\native-reader.exe"; IconFilename: "{app}\native-reader.ico"
Name: "{autodesktop}\{#AppName}"; Filename: "{app}\native-reader.exe"; IconFilename: "{app}\native-reader.ico"; Tasks: desktopicon

[Run]
Filename: "{app}\native-reader.exe"; Description: "启动 {#AppName}"; Flags: nowait postinstall skipifsilent

[UninstallRun]
Filename: "{app}\native-reader.exe"; Parameters: "--remove-associations"; Flags: runhidden waituntilterminated; RunOnceId: "RemoveNativeReaderAssociations"

[UninstallDelete]
Type: filesandordirs; Name: "{localappdata}\NativeReader"

[Code]
const
  RecommendedExtensions = ';.md;.markdown;.mdown;.mkd;.mdx;.html;.htm;.xhtml;.txt;.log;.csv;.tsv;.json;.jsonc;.yaml;.yml;.toml;.ini;.xml;.svg;.rs;.c;.h;.cc;.cpp;.hpp;.cs;.java;.kt;.js;.jsx;.mjs;.ts;.tsx;.py;.go;.sql;.sh;.ps1;.bat;.cmd;.css;.scss;.less;';

var
  AssocPage: TWizardPage;
  ModeAll: TRadioButton;
  ModeRecommended: TRadioButton;
  ModeCustom: TRadioButton;
  ModeNone: TRadioButton;
  AssocList: TNewCheckListBox;
  DefaultCheck: TCheckBox;
  AssocItems: TStringList;

function HasRecommended(const ExtList: String): Boolean;
var
  I, Start: Integer;
  Part: String;
begin
  Result := False;
  Start := 1;
  for I := 1 to Length(ExtList) + 1 do
    if (I > Length(ExtList)) or (ExtList[I] = ';') then
    begin
      Part := Trim(Copy(ExtList, Start, I - Start));
      if (Part <> '') and (Pos(';' + Part + ';', RecommendedExtensions) > 0) then
      begin
        Result := True;
        Exit;
      end;
      Start := I + 1;
    end;
end;

procedure AddAssocGroup(const Caption: String);
begin
  AssocList.AddGroup(Caption, '', 0, nil);
  AssocItems.Add('');
end;

procedure AddAssocItem(const Caption, ExtList: String);
begin
  AssocList.AddCheckBox(Caption, '', 1, False, True, False, True, nil);
  AssocItems.Add(ExtList);
end;

procedure SetChecks(UseRecommended: Boolean);
var
  I: Integer;
begin
  for I := 0 to AssocItems.Count - 1 do
    if AssocItems[I] <> '' then
    begin
      if UseRecommended then
        AssocList.Checked[I] := HasRecommended(AssocItems[I])
      else
        AssocList.Checked[I] := True;
    end;
end;

procedure UpdateMode(Sender: TObject);
begin
  if (AssocList = nil) or (DefaultCheck = nil) then
    Exit;
  if ModeAll.Checked then
  begin
    SetChecks(False);
    AssocList.Enabled := False;
    DefaultCheck.Enabled := True;
  end
  else if ModeRecommended.Checked then
  begin
    SetChecks(True);
    AssocList.Enabled := False;
    DefaultCheck.Enabled := True;
  end
  else if ModeCustom.Checked then
  begin
    AssocList.Enabled := True;
    DefaultCheck.Enabled := True;
  end
  else
  begin
    AssocList.Enabled := False;
    DefaultCheck.Enabled := False;
  end;
end;

function SelectedMode: String;
begin
  if ModeAll.Checked then
    Result := 'all'
  else if ModeRecommended.Checked then
    Result := 'recommended'
  else if ModeCustom.Checked then
    Result := 'custom'
  else
    Result := 'none';
end;

function SelectedExtensions: String;
var
  I: Integer;
begin
  Result := '';
  for I := 0 to AssocItems.Count - 1 do
    if (AssocItems[I] <> '') and AssocList.Checked[I] then
    begin
      if Result <> '' then
        Result := Result + ';';
      Result := Result + AssocItems[I];
    end;
end;

procedure InitializeWizard;
var
  Top: Integer;
begin
  AssocItems := TStringList.Create;
  AssocPage := CreateCustomPage(wpSelectTasks, '默认查看程序',
    '选择要接管的文本和源码类型。Windows 可能要求在系统设置中确认默认程序。');

  Top := 0;
  ModeAll := TRadioButton.Create(AssocPage);
  ModeAll.Parent := AssocPage.Surface;
  ModeAll.Left := 0;
  ModeAll.Top := Top;
  ModeAll.Width := AssocPage.SurfaceWidth;
  ModeAll.Caption := '全查看：接管全部支持的文本、代码和配置文件';
  ModeAll.OnClick := @UpdateMode;

  Top := Top + ScaleY(22);
  ModeRecommended := TRadioButton.Create(AssocPage);
  ModeRecommended.Parent := AssocPage.Surface;
  ModeRecommended.Left := 0;
  ModeRecommended.Top := Top;
  ModeRecommended.Width := AssocPage.SurfaceWidth;
  ModeRecommended.Caption := '默认查看：接管常用的 Markdown、HTML、文本和代码类型';
  ModeRecommended.Checked := True;
  ModeRecommended.OnClick := @UpdateMode;

  Top := Top + ScaleY(22);
  ModeCustom := TRadioButton.Create(AssocPage);
  ModeCustom.Parent := AssocPage.Surface;
  ModeCustom.Left := 0;
  ModeCustom.Top := Top;
  ModeCustom.Width := AssocPage.SurfaceWidth;
  ModeCustom.Caption := '用户自定义查看：在下方勾选需要接管的格式';
  ModeCustom.OnClick := @UpdateMode;

  Top := Top + ScaleY(22);
  ModeNone := TRadioButton.Create(AssocPage);
  ModeNone.Parent := AssocPage.Surface;
  ModeNone.Left := 0;
  ModeNone.Top := Top;
  ModeNone.Width := AssocPage.SurfaceWidth;
  ModeNone.Caption := '仅安装，不修改文件关联';
  ModeNone.OnClick := @UpdateMode;

  AssocList := TNewCheckListBox.Create(AssocPage);
  AssocList.Parent := AssocPage.Surface;
  AssocList.Left := 0;
  AssocList.Top := ModeNone.Top + ModeNone.Height + ScaleY(8);
  AssocList.Width := AssocPage.SurfaceWidth;
  AssocList.Height := ScaleY(126);
  AssocList.Flat := True;
  AssocList.TabStop := True;

  AddAssocGroup('文档与文本');
  AddAssocItem('Markdown (.md, .markdown, .mdown, .mkd, .mdx)', '.md;.markdown;.mdown;.mkd;.mdx');
  AddAssocItem('纯文本与日志 (.txt, .log)', '.txt;.log');
  AddAssocItem('表格文本 (.csv, .tsv)', '.csv;.tsv');
  AddAssocItem('文档标记 (.rst, .tex)', '.rst;.tex');

  AddAssocGroup('网页与样式');
  AddAssocItem('HTML 网页 (.html, .htm, .xhtml)', '.html;.htm;.xhtml');
  AddAssocItem('XML / SVG (.xml, .svg)', '.xml;.svg');
  AddAssocItem('样式表 (.css, .scss, .sass, .less)', '.css;.scss;.sass;.less');
  AddAssocItem('前端组件 (.vue, .svelte)', '.vue;.svelte');
  AddAssocItem('ASP.NET / XAML (.cshtml, .razor, .xaml)', '.cshtml;.razor;.xaml');

  AddAssocGroup('数据与配置');
  AddAssocItem('JSON (.json, .jsonc)', '.json;.jsonc');
  AddAssocItem('YAML / TOML (.yaml, .yml, .toml)', '.yaml;.yml;.toml');
  AddAssocItem('INI / 配置 (.ini, .cfg, .conf, .config, .properties)', '.ini;.cfg;.conf;.config;.properties');
  AddAssocItem('API / Schema (.graphql, .gql, .proto)', '.graphql;.gql;.proto');

  AddAssocGroup('编程语言');
  AddAssocItem('Rust / C / C++ (.rs, .c, .h, .cc, .cpp, .cxx, .hpp, .hxx, .hh)', '.rs;.c;.h;.cc;.cpp;.cxx;.hpp;.hxx;.hh');
  AddAssocItem('C# / Java / Kotlin / Scala (.cs, .java, .kt, .kts, .scala)', '.cs;.java;.kt;.kts;.scala');
  AddAssocItem('JavaScript / TypeScript (.js, .jsx, .mjs, .cjs, .ts, .tsx, .mts, .cts)', '.js;.jsx;.mjs;.cjs;.ts;.tsx;.mts;.cts');
  AddAssocItem('Python / Go / SQL (.py, .pyw, .pyi, .go, .sql)', '.py;.pyw;.pyi;.go;.sql');
  AddAssocItem('Shell / PowerShell / 批处理 (.sh, .bash, .zsh, .fish, .ps1, .psm1, .psd1, .bat, .cmd)', '.sh;.bash;.zsh;.fish;.ps1;.psm1;.psd1;.bat;.cmd');
  AddAssocItem('其他语言 (.lua, .rb, .php, .pl, .pm, .swift, .dart, .r, .asm, .s, .vb, .vbs)', '.lua;.rb;.php;.pl;.pm;.swift;.dart;.r;.asm;.s;.vb;.vbs');

  AddAssocGroup('项目文件');
  AddAssocItem('Visual Studio 项目 (.csproj, .vcxproj, .props, .targets, .sln)', '.csproj;.vcxproj;.props;.targets;.sln');

  SetChecks(True);

  DefaultCheck := TCheckBox.Create(AssocPage);
  DefaultCheck.Parent := AssocPage.Surface;
  DefaultCheck.Left := 0;
  DefaultCheck.Top := AssocList.Top + AssocList.Height + ScaleY(8);
  DefaultCheck.Width := AssocPage.SurfaceWidth;
  DefaultCheck.Caption := '安装后尝试设为这些类型的默认打开程序（Windows 可能要求确认）';
  DefaultCheck.Checked := True;
  UpdateMode(nil);
end;

function NextButtonClick(CurPageID: Integer): Boolean;
begin
  Result := True;
  if (AssocPage <> nil) and (CurPageID = AssocPage.ID) then
    if ModeCustom.Checked and (SelectedExtensions = '') then
    begin
      MsgBox('请至少勾选一种文件类型，或选择“仅安装，不修改文件关联”。', mbError, MB_OK);
      Result := False;
    end;
end;

procedure CurStepChanged(CurStep: TSetupStep);
var
  Mode: String;
  ResultCode, ErrorCode: Integer;
begin
  if CurStep = ssPostInstall then
  begin
    Mode := SelectedMode;
    RegWriteStringValue(HKCU, 'Software\NativeReader\Associations', 'Mode', Mode);
    RegWriteStringValue(HKCU, 'Software\NativeReader\Associations', 'Extensions', SelectedExtensions);
    RegWriteDWordValue(HKCU, 'Software\NativeReader\Associations', 'SetDefault',
      Ord(DefaultCheck.Checked and (Mode <> 'none')));
    RegWriteDWordValue(HKCU, 'Software\NativeReader\Associations', 'Prompted', 1);

    if not Exec(ExpandConstant('{app}\native-reader.exe'), '--apply-associations', '',
      SW_HIDE, ewWaitUntilTerminated, ResultCode) then
      MsgBox('无法应用文件关联设置。', mbError, MB_OK)
    else if (Mode <> 'none') and DefaultCheck.Checked and (ResultCode <> 0) then
    begin
      if MsgBox('Native Reader 已注册为可选打开方式，' + #13#10 +
        '但 Windows 保护了部分现有默认程序。' + #13#10 + #13#10 +
        '是否现在打开“默认应用”设置进行确认？', mbConfirmation, MB_YESNO) = IDYES then
        ShellExec('open', 'ms-settings:defaultapps?registeredAppUser=Native%20Reader', '',
          '', SW_SHOWNORMAL, ewNoWait, ErrorCode);
    end;
  end;
end;

procedure DeinitializeSetup;
begin
  if AssocItems <> nil then
    AssocItems.Free;
end;
