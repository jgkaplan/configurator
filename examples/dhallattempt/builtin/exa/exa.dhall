let shell-escape = https://prelude.dhall-lang.org/Text/shell-escape
let concatSep = https://prelude.dhall-lang.org/Text/concatSep
let Map = https://prelude.dhall-lang.org/Map/Type

let Config : Type = {
    enable: Bool,
    enableAliases: Bool,
    extraOptions: List Text,
    icons: Bool,
    git: Bool
}

let Output : Type = {
    install: List Text,
    aliases: Map Text Text
}

let Defaults = {
    Type = Config,
    default = {
        enable = False,
        enableAliases = True,
        extraOptions = [],
        icons = False,
        git = False
    }
}

let aliases = {
    ls = "exa",
    ll = "exa -l",
    la = "exa -a",
    lt = "exa --tree",
    lla = "exa -la"
}

let makeOutput : Config -> Output = \(c: Config) -> 
    let merged = Defaults::c
    let icons = if merged.icons then "--icons" else ""
    let git = if merged.git then "--git" else ""
    let args = shell-escape ("" ++ icons ++ git ++ (concatSep " " merged.extraOptions))
    let command = "exa ${args}"
    in {
        install = ["exa"],
        aliases = toMap ({
            exa = command
        } /\ (if merged.enableAliases then aliases else {=}))
    }

in {
    Input = Config,
    Output = Output,
    fun = makeOutput
}