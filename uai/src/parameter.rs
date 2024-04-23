#[derive(Clone)]
pub enum Parameter {
    /// Check represents a checkbox parameter which can be true or false.
    ///
    /// Its first field contains the default value for the checkbox.
    Check(bool),

    /// String represents a string parameter which can have any string value.
    ///
    /// Its first field contains the default String value for the parameter.
    String(String),

    /// Spin represents a spin wheel which can be an integer in a certain range.
    ///
    /// The first argument is the default value, the second argument is the minimum
    /// value, and the third argument is the maximum value of this parameter. The
    /// minimum and maximum bounds are inclusive and and the default must be in
    /// the range defined by them.
    Spin(i32, i32, i32),

    /// Combo represents a combo box which can have the value of one of the
    /// predefined strings.
    ///
    /// Its first argument is the default value while the second argument is the
    /// list of predefined strings. The default value must be included in the list.
    Combo(String, Vec<String>),
}
