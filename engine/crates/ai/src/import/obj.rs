//! The PostScript objects the interpreter works with, its operators and its errors.


#[cfg(test)]
thread_local! {
    /// Tests: the arrays of the list last released on this thread that were still alive after it
    /// (the ones the interpreter itself still held).
    pub(crate) static ARRAYS_LEFT: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
    /// Tests: how many values [`Shared::to_vec`] has copied on this thread.
    pub(crate) static VALUES_COPIED: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}


macro_rules! ops {
    ($($v:ident = $n:literal),* $(,)?) => {

    };
}

ops! {
    // Stack.
    Pop = "pop", Exch = "exch", Dup = "dup", Copy = "copy", Index = "index", Roll = "roll", Clear = "clear", Count = "count",
    Mark = "mark", MarkBracket = "[", CloseArray = "]", DictBegin = "<<", DictEnd = ">>", ClearToMark = "cleartomark",
    CountToMark = "counttomark",
    // Arithmetic.
    Add = "add", Sub = "sub", Mul = "mul", Div = "div", Idiv = "idiv", Mod = "mod", Neg = "neg", Abs = "abs", Sqrt = "sqrt",
    Sin = "sin", Cos = "cos", Atan = "atan", Exp = "exp", Ln = "ln", Log = "log", Round = "round", Floor = "floor",
    Ceiling = "ceiling", Truncate = "truncate", Cvi = "cvi", Cvr = "cvr", Rand = "rand", Srand = "srand", Rrand = "rrand",
    // Relations and logic.
    Eq = "eq", Ne = "ne", Gt = "gt", Ge = "ge", Lt = "lt", Le = "le", And = "and", Or = "or", Xor = "xor", Not = "not",
    Bitshift = "bitshift", True = "true", False = "false", Null = "null",
    // Control.
    Exec = "exec", If = "if", IfElse = "ifelse", For = "for", Repeat = "repeat", Loop = "loop", Exit = "exit",
    Forall = "forall", Stopped = "stopped", Stop = "stop", Quit = "quit", CountExecStack = "countexecstack",
    // Types.
    Type = "type", Cvx = "cvx", Cvlit = "cvlit", Xcheck = "xcheck", Cvn = "cvn", Cvs = "cvs", Cvrs = "cvrs",
    ReadOnly = "readonly", ExecuteOnly = "executeonly", NoAccess = "noaccess", Rcheck = "rcheck", Wcheck = "wcheck",
    // Composites.
    NewArray = "array", PackedArray = "packedarray", SetPacking = "setpacking", CurrentPacking = "currentpacking",
    Length = "length", Get = "get", Put = "put", GetInterval = "getinterval", PutInterval = "putinterval",
    Aload = "aload", Astore = "astore", NewString = "string", Search = "search", AnchorSearch = "anchorsearch",
    // Dictionaries.
    NewDict = "dict", MaxLength = "maxlength", Begin = "begin", End = "end", Def = "def", Load = "load", Store = "store",
    Known = "known", Where = "where", Undef = "undef", CurrentDict = "currentdict", CountDictStack = "countdictstack",
    DictStack = "dictstack", SystemDict = "systemdict", UserDict = "userdict", GlobalDict = "globaldict", StatusDict = "statusdict",
    ErrorDict = "errordict", DollarError = "$error", InternalDict = "internaldict", Bind = "bind",
    // Virtual memory and the environment.
    Save = "save", Restore = "restore", SetGlobal = "setglobal", CurrentGlobal = "currentglobal", VmStatus = "vmstatus",
    Version = "version", Product = "product", RealTime = "realtime", UserTime = "usertime",
    Print = "print", EqPrint = "=", EqEqPrint = "==", Pstack = "pstack", Stack = "stack", Flush = "flush",
    // Graphics state.
    Gsave = "gsave", Grestore = "grestore", GrestoreAll = "grestoreall", InitGraphics = "initgraphics",
    GStateNew = "gstate", CurrentGState = "currentgstate", SetGState = "setgstate",
    SetLineWidth = "setlinewidth", CurrentLineWidth = "currentlinewidth", SetLineCap = "setlinecap",
    CurrentLineCap = "currentlinecap", SetLineJoin = "setlinejoin", CurrentLineJoin = "currentlinejoin",
    SetMiterLimit = "setmiterlimit", CurrentMiterLimit = "currentmiterlimit", SetDash = "setdash", CurrentDash = "currentdash",
    SetGray = "setgray", CurrentGray = "currentgray", SetRgbColor = "setrgbcolor", CurrentRgbColor = "currentrgbcolor",
    SetHsbColor = "sethsbcolor", SetCmykColor = "setcmykcolor", CurrentCmykColor = "currentcmykcolor",
    SetColorSpace = "setcolorspace", CurrentColorSpace = "currentcolorspace", SetColor = "setcolor", CurrentColor = "currentcolor",
    SetPattern = "setpattern", MakePattern = "makepattern", SetOverprint = "setoverprint", CurrentOverprint = "currentoverprint",
    // Device settings without an effect on the art.
    SetFlat = "setflat", CurrentFlat = "currentflat", SetScreen = "setscreen", SetHalftone = "sethalftone",
    SetTransfer = "settransfer", SetColorTransfer = "setcolortransfer", SetBlackGeneration = "setblackgeneration",
    SetUnderColorRemoval = "setundercolorremoval", SetColorRendering = "setcolorrendering", SetSmoothness = "setsmoothness",
    SetStrokeAdjust = "setstrokeadjust", CurrentStrokeAdjust = "currentstrokeadjust", SetPageDevice = "setpagedevice",
    CurrentPageDevice = "currentpagedevice", SetUserParams = "setuserparams", SetSystemParams = "setsystemparams",
    CurrentUserParams = "currentuserparams", CurrentSystemParams = "currentsystemparams",
    SetObjectFormat = "setobjectformat", ShowPage = "showpage", CopyPage = "copypage", ErasePage = "erasepage",
    CurrentScreen = "currentscreen", SetColorScreen = "setcolorscreen", CurrentHalftone = "currenthalftone",
    CurrentTransfer = "currenttransfer", CurrentColorTransfer = "currentcolortransfer",
    CurrentBlackGeneration = "currentblackgeneration", CurrentUnderColorRemoval = "currentundercolorremoval",
    SetCacheLimit = "setcachelimit", UCache = "ucache", SetUCacheParams = "setucacheparams", FindEncoding = "findencoding",
    // Coordinates.
    Matrix = "matrix", IdentMatrix = "identmatrix", DefaultMatrix = "defaultmatrix", CurrentMatrix = "currentmatrix",
    SetMatrix = "setmatrix", InitMatrix = "initmatrix", Concat = "concat", ConcatMatrix = "concatmatrix",
    Translate = "translate", Scale = "scale", Rotate = "rotate", Transform = "transform", ITransform = "itransform",
    DTransform = "dtransform", IDTransform = "idtransform", InvertMatrix = "invertmatrix",
    // Paths and painting.
    NewPath = "newpath", CurrentPoint = "currentpoint", MoveTo = "moveto", RMoveTo = "rmoveto", LineTo = "lineto",
    RLineTo = "rlineto", CurveTo = "curveto", RCurveTo = "rcurveto", Arc = "arc", Arcn = "arcn", Arct = "arct",
    Arcto = "arcto", ClosePath = "closepath", FlattenPath = "flattenpath", ReversePath = "reversepath",
    PathBBox = "pathbbox", ClipPath = "clippath", InitClip = "initclip", Clip = "clip", EoClip = "eoclip",
    RectClip = "rectclip", ClipSave = "clipsave", ClipRestore = "cliprestore", Fill = "fill", EoFill = "eofill",
    Stroke = "stroke", RectFill = "rectfill", RectStroke = "rectstroke", ShFill = "shfill",
    // Images.
    Image = "image", ImageMask = "imagemask", ColorImage = "colorimage",
    // Files.
    CurrentFile = "currentfile", Filter = "filter", ReadHexString = "readhexstring", ReadString = "readstring",
    ReadLine = "readline", FlushFile = "flushfile", CloseFile = "closefile", File = "file", Eexec = "eexec",
    // Fonts and type.
    FindFont = "findfont", ScaleFont = "scalefont", MakeFont = "makefont", SetFont = "setfont", SelectFont = "selectfont",
    CurrentFont = "currentfont", DefineFont = "definefont", UndefineFont = "undefinefont", FindResource = "findresource",
    DefineResource = "defineresource", UndefineResource = "undefineresource", ResourceStatus = "resourcestatus",
    ResourceForAll = "resourceforall", Show = "show", AShow = "ashow",
    WidthShow = "widthshow", AWidthShow = "awidthshow", XShow = "xshow", YShow = "yshow", XYShow = "xyshow",
    KShow = "kshow", CShow = "cshow", GlyphShow = "glyphshow", StringWidth = "stringwidth", CharPath = "charpath",
    SetCacheDevice = "setcachedevice", SetCharWidth = "setcharwidth", SetCacheDevice2 = "setcachedevice2", RootFont = "rootfont",
    // Forms, devices and files programs probe (no output device or file system here).
    ExecForm = "execform", NullDevice = "nulldevice", PdfMark = "pdfmark", Status = "status", Token = "token",
    BytesAvailable = "bytesavailable", ResetFile = "resetfile", Write = "write", WriteString = "writestring",
    WriteHexString = "writehexstring", Echo = "echo", StrokePath = "strokepath", PathForAll = "pathforall",
    // Queries and settings without an effect on the art.
    CurrentHsbColor = "currenthsbcolor", CurrentColorRendering = "currentcolorrendering",
    FindColorRendering = "findcolorrendering", CurrentColorScreen = "currentcolorscreen", CurrentSmoothness = "currentsmoothness",
    SetHalftonePhase = "sethalftonephase", CurrentHalftonePhase = "currenthalftonephase", SetDevParams = "setdevparams",
    CurrentDevParams = "currentdevparams", Gcheck = "gcheck", Scheck = "scheck", CurrentShared = "currentshared",
    SetShared = "setshared", ClearDictStack = "cleardictstack", ExecStack = "execstack", SetCacheParams = "setcacheparams",
    CurrentCacheParams = "currentcacheparams", CacheStatus = "cachestatus", UCacheStatus = "ucachestatus",
    SetVmThreshold = "setvmthreshold", VmReclaim = "vmreclaim", StartJob = "startjob", CurrentObjectFormat = "currentobjectformat",
    Revision = "revision", SerialNumber = "serialnumber", Read = "read", FilePosition = "fileposition",
    SetFilePosition = "setfileposition", DeleteFile = "deletefile", RenameFile = "renamefile", Run = "run",
    // CID-keyed fonts: `composefont`, and the operators of the CIDInit procedure set (CMaps and
    // `StartData`, which FontSetInit has too), which aren't in systemdict (see `Op::PROCSET`).
    ComposeFont = "composefont", BeginCMap = "begincmap", EndCMap = "endcmap", UseCMap = "usecmap",
    BeginCodeSpaceRange = "begincodespacerange", EndCodeSpaceRange = "endcodespacerange", UseFont = "usefont",
    BeginBfChar = "beginbfchar", EndBfChar = "endbfchar", BeginBfRange = "beginbfrange", EndBfRange = "endbfrange",
    BeginCidChar = "begincidchar", EndCidChar = "endcidchar", BeginCidRange = "begincidrange", EndCidRange = "endcidrange",
    BeginNotDefChar = "beginnotdefchar", EndNotDefChar = "endnotdefchar", BeginNotDefRange = "beginnotdefrange",
    EndNotDefRange = "endnotdefrange", BeginRearrangedFont = "beginrearrangedfont", EndRearrangedFont = "endrearrangedfont",
    BeginUseMatrix = "beginusematrix", EndUseMatrix = "endusematrix", StartData = "StartData",
    // User paths.
    UFill = "ufill", UEoFill = "ueofill", UStroke = "ustroke", UAppend = "uappend", UPath = "upath", SetBBox = "setbbox",
}
