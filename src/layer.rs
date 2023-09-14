#[macro_export]
macro_rules! create_layer {
    (
     over $($sublayer:ident),*
     | atoms $($atom:ident$(<$generic:tt>)?),*
     | builtins $($name:literal -> $value:expr),*
    ) => {
        macro_rules! add_my_atoms {
           ( | $$($$unfolded_layer:ident)* | $$($$atom:tt$$(<$$generic:tt>)?),* ) => {
             create_expression!(
               Expression,
               $$($$atom$$(<$$generic>)?,)*
               List<Expression>,
               BuiltinFunction<Expression>,
               BuiltinMacro<Expression>,
               Lambda<Expression>,
               Macro<Expression>,
               Number,
               Symbol
             );

             pub fn set_environment<Expression: $$(ToAndFrom<$$atom$$(<$$generic>)?> + )* LispExpression>(env: &mut Environment<Expression>) {
                $$(
                $$unfolded_layer::set_environment(env);
                )*
                // TODO reintroduce builtins of this layer
             }
           };
           ($$($$layer:ident)+ | $$($$unfolded_layer:ident)+ | $$($$atom:tt$$(<$$generic:tt>)?),* ) => {
             other_crate::add_my_atoms( $($atom$(<$generic>)?)*);
             todo!()
           }
        }

        add_my_atoms!( | $($sublayer)* | $($atom$(<$generic>)?),*);
    };
}
