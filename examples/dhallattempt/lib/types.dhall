let OutputType = <JSON | INI | TOML | YAML | TEXT>

let Config : Type = {
    -- I don't really like this. it would be nice if it generalized
    -- and wasn't specific to just things to be added to terminal config files
    -- althought, I'm not sure any other case is necessary
    aliases : List Text, -- to be added to terminal config
    install : List Text
}

let Module : Type = {
    Input: Type,
    Output: Type,
    fun: Input -> Output -- this probably won't work. might need to parameterize on types :(
}

-- this doesn't force the Input in the record to be the passed in input type
-- let Module = \(Input: Type) -> \(Output : Type) -> {
--     Input: Type,
--     Output: Type,
--     fun: Input -> Output
-- }

let Operations = <
      Command: Text 
    | WriteFile: {filename: Text, type: OutputType, contents: forall (a : Type) -> a{- hmmm -}}
    | Wait: {} -- More information is needed, so wait for everything else to process
    >
    -- \(a : Type) -> \(x : a) -> x