#[macro_export]
macro_rules! create_layer {
    (
     over $first_sublayer:ident $($other_sublayer:ident)*
     | atoms $($atom:ident$(<$generic:tt>)?),*
     | builtins $($name:literal -> $value:expr),*
    ) => {
        #[macro_export]
        macro_rules! add_my_atoms {
           (
             $$top_layer:ident $$($$layer:ident)* ;  // Layers to go
             $$($$unfolded_layer:ident)* ;           // Layers already done
             $$($$atom:ident$$(<$$generic:tt>)?,)* ; // Atoms in the done layers
             $$($$name:literal -> $$value:expr)*     // Builtins of the top layer
           ) => {
             $$top_layer::add_my_atoms!(
               $$($$layer)* $first_sublayer $($other_sublayer)*;
               $$top_layer $$($$unfolded_layer)* ;
               $($atom$(<$generic>)?,)* $$($$atom$$(<$$generic>)?,)* ;
               $$($$name -> $$value)* // Builtins of the top layer
             );
           };
           (
             ;  // Layers to go
             $$($$unfolded_layer:ident)* ;           // Layers already done
             $$($$atom:ident$$(<$$generic:tt>)?,)* ; // Atoms in the done layers
             $$($$name:literal -> $$value:expr)*     // Builtins of the top layer
           ) => {
             $first_sublayer::add_my_atoms!(
               $($other_sublayer)*;
               $first_sublayer $$($$unfolded_layer)* ;
               $($atom$(<$generic>)?,)* $$($$atom$$(<$$generic>)?,)* ;
               $$($$name -> $$value)* // Builtins of the top layer
             );
           }
        }

        $first_sublayer::add_my_atoms!(
          $($other_sublayer)*;
          $first_sublayer ;
          $($atom$(<$generic>)?,)* ;
          $($name -> $value)*
        );
    };

    (
     atoms $($atom:ident$(<$generic:tt>)?),*
     | builtins $($name:literal -> $value:expr),*
    ) => {
        #[macro_export]
        macro_rules! add_my_atoms {
           (
            ; // Layers to go
            $$($$unfolded_layer:ident)* ; // Layers already done
            $$($$atom:ident$$(<$$generic:tt>)?,)* ; // Atoms in the done layers
            $$($$name:literal -> $$value:expr)* // Builtins of the top layer
           ) => {
             create_expression!(
               Expression,
               $($atom$(<$generic>)?,)*
               $$($$atom$$(<$$generic>)?,)*
               List<Expression>,
               BuiltinFunction<Expression>,
               BuiltinMacro<Expression>,
               Lambda<Expression>,
               Macro<Expression>,
               Number,
               Symbol
             );

             pub fn set_environment<Expression>(env: &mut Environment<Expression>)
             where
                Expression:
                    $(ToAndFrom<$atom$(<$generic>)?> + )*
                    $$(ToAndFrom<$$atom$$(<$$generic>)?> + )*
                    LispExpression
             {
                $$(
                $$unfolded_layer::set_environment(env);
                )*
                $$(
                env.set($$name, $$value);
                )*
             }
           };
           (
             $$top_layer:ident $$($$layer:ident)* ;  // Layers to go
             $$($$unfolded_layer:ident)* ;           // Layers already done
             $$($$atom:ident$$(<$$generic:tt>)?,)* ; // Atoms in the done layers
             $$($$name:literal -> $$value:expr)*     // Builtins of the top layer
           ) => {
             $$top_layer::add_my_atoms!(
               $$($$layer)*;
               $$top_layer $$($$unfolded_layer)* ;
               $($atom$(<$generic>)?,)* $$($$atom$$(<$$generic>)?,)* ;
               $$($$name -> $$value)* // Builtins of the top layer
             );
           }
        }

        add_my_atoms!( ; ; ; $($name -> $value)*);
    };
}
