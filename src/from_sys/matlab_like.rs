use crate::from_sys::{
    FromSys,
    script::{MatParser, ScriptInspector},
};
use std::collections::HashSet;

pub struct MatLabLikeBuilder {
    pub target: String,
}

impl MatLabLikeBuilder {
    pub fn build(self) -> FromSys {
        let start_out_block = FromSys::get_start_out_block();
        let end_out_block = FromSys::get_end_out_block();
        FromSys {
            base_command: self.target,
            print_value_pattern: format!(
                r#"
printf("{start_out_block}")
vars = whos;

allVars = struct();

for k = 1:length(vars)
    name = vars(k).name;
    value = evalin('caller', name);
    
    if isnumeric(value) || islogical(value) || ischar(value) || isstring(value) || ...
       isstruct(value) || iscell(value)
        allVars.(name) = value;
        
    elseif istable(value)
        allVars.(name) = table2struct(value);
        
    elseif isdatetime(value) || isduration(value) || iscategorical(value)
        allVars.(name) = string(value);
        
    elseif iscomplex(value)
        allVars.(name) = struct('re', real(value), 'im', imag(value));
        
    elseif issparse(value)
        [i,j,s] = find(value);
        allVars.(name) = struct('i', i, 'j', j, 'value', s, 'size', size(value));
    else
        allVars.(name) = sprintf('<%s>', class(value));
    end
end

jsonString = jsonencode(allVars, 'PrettyPrint', true);
disp(jsonString);

printf("{end_out_block}")
            "#
            ),
            input_value_pattern: "input_data = jsondecode({});".to_string(),
            script_inspector: ScriptInspector {
                restricted_functions: HashSet::new(),
                parser: Box::new(MatParser {}) as _,
            },
        }
    }
}

#[cfg(test)]
mod test {
    use serde_json::{Number, Value as JsonValue};

    use crate::run_script::RunScript;
    use std::collections::HashMap;

    use super::*;

    #[test]
    fn simle() {
        let mut data = HashMap::new();
        data.insert(
            "test_value".to_string(),
            JsonValue::Number(Number::from_u128(42).unwrap()),
        );

        let script_result = MatLabLikeBuilder {
            target: "octave".into(),
        }
        .build()
        .run_script("input_data.test_value ^ 2".to_string().into(), data)
        .unwrap()
        .get_result()
        .as_u64()
        .unwrap();

        assert_eq!(script_result, 1764);
    }
}
