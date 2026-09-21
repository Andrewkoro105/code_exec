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
                parser: Box::new(MatParser) as _,
            },
            init_script: None,
            runner: None,
        }
    }
}

#[cfg(test)]
mod test {
    use serde_json::{Number, Value as JsonValue};

    use crate::{
        clean::Clean,
        from_sys::{FromSysError, runner},
        run::Run,
        run_script::RunScript,
        set_init_script::SetInitScript,
    };
    use std::collections::HashMap;

    use super::*;

    #[tokio::test]
    async fn run_script() {
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
        .await
        .unwrap()
        .get_result()
        .as_u64()
        .unwrap();

        assert_eq!(script_result, 1764);
    }

    
    #[tokio::test]
    async fn init_run_script() -> Result<(), FromSysError> {
        let mut data = HashMap::new();
        data.insert(
            "test_value".to_string(),
            JsonValue::Number(Number::from_u128(42).unwrap()),
        );

        let mut octave = MatLabLikeBuilder {
            target: "octave".into(),
        }
        .build();
        octave.set_init_script(
            "input_data.test_value = input_data.test_value ^ 2"
                .to_string()
                .into(),
            data,
        )?;

        let result = octave
            .run_script(
                "input_data.test_value + 2".to_string().into(),
                HashMap::new(),
            )
            .await
            .unwrap()
            .get_result()
            .as_u64()
            .unwrap();
        assert_eq!(result, 42u64.pow(2) + 2);

        let result = octave
            .run_script(
                "input_data.test_value * 2".to_string().into(),
                HashMap::new(),
            )
            .await
            .unwrap()
            .get_result()
            .as_u64()
            .unwrap();
        assert_eq!(result, 42u64.pow(2) * 2);
        Ok(())
    }

    
    #[tokio::test]
    async fn run() -> Result<(), FromSysError> {
        let mut data = HashMap::new();
        data.insert(
            "test_value".to_string(),
            JsonValue::Number(Number::from_u128(42).unwrap()),
        );

        let mut octave = MatLabLikeBuilder {
            target: "octave".into(),
        }
        .build();
        octave.run(
            "input_data.test_value = input_data.test_value * 2"
                .to_string()
                .into(),
            data,
        ).await?;
        let result = octave
            .run(
                "input_data.test_value + 2".to_string().into(),
                HashMap::new(),
            ).await?
            .get_result()
            .as_u64()
            .unwrap();
        assert_eq!(result, 42 * 2 + 2);

        Ok(())
    }

    
    #[tokio::test]
    async fn init_run() -> Result<(), FromSysError> {
        let mut data = HashMap::new();
        data.insert(
            "test_value".to_string(),
            JsonValue::Number(Number::from_u128(42).unwrap()),
        );

        let mut octave = MatLabLikeBuilder {
            target: "octave".into(),
        }
        .build();
        octave.set_init_script(
            "input_data.test_value = input_data.test_value ^ 2"
                .to_string()
                .into(),
            data,
        )?;
        octave.run(
            "input_data.test_value = input_data.test_value * 2"
                .to_string()
                .into(),
            HashMap::new(),
        ).await?;
        let result = octave
            .run(
                "input_data.test_value + 2".to_string().into(),
                HashMap::new(),
            ).await?
            .get_result()
            .as_u64()
            .unwrap();
        assert_eq!(result, 42u64.pow(2) * 2 + 2);

        octave.clean().await?;
        let result = octave
            .run(
                "input_data.test_value + 2".to_string().into(),
                HashMap::new(),
            ).await?
            .get_result()
            .as_u64()
            .unwrap();
        assert_eq!(result, 42u64.pow(2) + 2);

        Ok(())
    }

    
    #[tokio::test]
    async fn error() {
        let script_result = MatLabLikeBuilder {
            target: "octave".into(),
        }
        .build()
        .run_script(
            "input_data.test_value ^ 2".to_string().into(),
            HashMap::new(),
        )
        .await;

        match script_result {
            Err(FromSysError::Runner(runner::Error::ExitStatus(_, err))) => {assert_eq!(err, "error: 'input_data' undefined near line 1, column 27\n")},
            _ => panic!("{script_result:?} != Err(FromSysError::Runner(runner::Error::ExitStatus(_, _)))")
        }
    }
}
