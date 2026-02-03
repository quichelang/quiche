# Parsley - Argument Parser for Quiche
#
# Features:
# - Default arguments
# - Shorthands (-v for --verbose)
# - Help text generation
# - Type-safe result parsing

from std.collections import HashMap

class ParseResult:
    flags: HashMap[String, bool]
    options: HashMap[String, String]
    positionals: Vec[String]
    errors: Vec[String]

    def new() -> ParseResult:
        return ParseResult(
            flags=HashMap.new(),
            options=HashMap.new(),
            positionals=[],
            errors=[]
        )

    def get_flag(self, name: String) -> bool:
        match self.flags.get(ref(name)):
            case Some(v):
                return deref(v)
            case None:
                return False

    def get_option(self, name: String) -> String:
        match self.options.get(ref(name)):
            case Some(v):
                return deref(v).clone()
            case None:
                return ""

    def has_errors(self) -> bool:
        return self.errors.len() > 0


class ArgSpec:
    name: String
    short: String
    description: String
    default_value: String
    is_flag: bool
    takes_value: bool
    required: bool
    is_positional: bool

    def new_flag(name: String, short: String, description: String) -> ArgSpec:
        return ArgSpec(
            name=name,
            short=short,
            description=description,
            default_value="false",
            is_flag=True,
            takes_value=False,
            required=False,
            is_positional=False
        )

    def new_option(name: String, short: String, description: String, default: String, required: bool) -> ArgSpec:
        return ArgSpec(
            name=name,
            short=short,
            description=description,
            default_value=default,
            is_flag=False,
            takes_value=True,
            required=required,
            is_positional=False
        )

    def new_positional(name: String, description: String, required: bool) -> ArgSpec:
        return ArgSpec(
            name=name,
            short="",
            description=description,
            default_value="",
            is_flag=False,
            takes_value=True,
            required=required,
            is_positional=True
        )


class Parser:
    program_name: String
    description: String
    _specs: Vec[ArgSpec]           # Private - use add_flag/add_option
    _positional_specs: Vec[ArgSpec] # Private - use add_positional

    def new(program_name: String, description: String) -> Parser:
        return Parser(
            program_name=program_name,
            description=description,
            _specs=[],
            _positional_specs=[]
        )

    def add_flag(self, name: String, short: String, description: String):
        spec = ArgSpec.new_flag(name, short, description)
        self._specs.push(spec)

    def add_option(self, name: String, short: String, description: String, default: String):
        spec = ArgSpec.new_option(name, short, description, default, False)
        self._specs.push(spec)

    def add_required_option(self, name: String, short: String, description: String):
        spec = ArgSpec.new_option(name, short, description, "", True)
        self._specs.push(spec)

    def add_positional(self, name: String, description: String):
        spec = ArgSpec.new_positional(name, description, True)
        self._positional_specs.push(spec)

    def add_optional_positional(self, name: String, description: String):
        spec = ArgSpec.new_positional(name, description, False)
        self._positional_specs.push(spec)

    def parse(self, args: Vec[String]) -> ParseResult:
        result = ParseResult.new()

        # Initialize defaults for flags
        for spec in self._specs.iter():
            if spec.is_flag:
                result.flags.insert(spec.name.clone(), False)
            elif spec.default_value != "":
                result.options.insert(spec.name.clone(), spec.default_value.clone())

        i = 0
        positional_idx = 0

        while i < args.len():
            arg = args[i].clone()

            if arg.starts_with(ref("--")):
                # Long option
                name = arg[2..].to_string()
                found = False

                for spec in self._specs.iter():
                    if spec.name == name:
                        found = True
                        if spec.is_flag:
                            result.flags.insert(name.clone(), True)
                        elif spec.takes_value:
                            if i + 1 < args.len():
                                i = i + 1
                                result.options.insert(name.clone(), args[i].clone())
                            else:
                                result.errors.push("Missing value for --" + name.as_str())
                        break

                if not found:
                    result.errors.push("Unknown option: --" + name.as_str())

            elif arg.starts_with(ref("-")) and arg.len() > 1:
                # Short option
                short = arg[1..2].to_string()
                found = False

                for spec in self._specs.iter():
                    if spec.short == short:
                        found = True
                        if spec.is_flag:
                            result.flags.insert(spec.name.clone(), True)
                        elif spec.takes_value:
                            # Check for attached value (-ovalue)
                            if arg.len() > 2:
                                result.options.insert(spec.name.clone(), arg[2..].to_string())
                            elif i + 1 < args.len():
                                i = i + 1
                                result.options.insert(spec.name.clone(), args[i].clone())
                            else:
                                result.errors.push("Missing value for -" + short.as_str())
                        break

                if not found:
                    result.errors.push("Unknown option: -" + short.as_str())
            else:
                # Positional argument
                result.positionals.push(arg)
                positional_idx = positional_idx + 1

            i = i + 1

        # Check required options
        for spec in self._specs.iter():
            if spec.required:
                match result.options.get(ref(spec.name)):
                    case Some(_):
                        pass
                    case None:
                        result.errors.push("Required option missing: --" + spec.name.as_str())

        # Check required positionals
        pi = 0
        while pi < self._positional_specs.len():
            spec = self._positional_specs[pi].clone()
            if spec.required and pi >= result.positionals.len():
                result.errors.push("Required argument missing: " + spec.name.as_str())
            pi = pi + 1

        return result

    def help_text(self) -> String:
        out = self.program_name.clone() + "\n"
        if self.description != "":
            out = out + self.description.as_str() + "\n"
        out = out + "\nUSAGE:\n"
        out = out + "    " + self.program_name.as_str()

        # Show positionals in usage
        for spec in self._positional_specs.iter():
            if spec.required:
                out = out + " <" + spec.name.as_str() + ">"
            else:
                out = out + " [" + spec.name.as_str() + "]"

        out = out + " [OPTIONS]\n\n"

        # Positional arguments
        if self._positional_specs.len() > 0:
            out = out + "ARGS:\n"
            for spec in self._positional_specs.iter():
                out = out + "    <" + spec.name.as_str() + ">  " + spec.description.as_str() + "\n"
            out = out + "\n"

        # Options
        out = out + "OPTIONS:\n"
        for spec in self._specs.iter():
            line = "    "
            if spec.short != "":
                line = line + "-" + spec.short.as_str() + ", "
            else:
                line = line + "    "
            line = line + "--" + spec.name.as_str()
            if spec.takes_value:
                line = line + " <VALUE>"

            # Pad to alignment
            while line.len() < 30:
                line = line + " "

            line = line + spec.description.as_str()

            if spec.default_value != "" and not spec.is_flag:
                line = line + " [default: " + spec.default_value.as_str() + "]"

            if spec.required:
                line = line + " [required]"

            out = out + line.as_str() + "\n"

        out = out + "    -h, --help                Help message\n"

        return out