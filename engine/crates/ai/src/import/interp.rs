//! The interpreter: the operand and dictionary stacks, running objects and procedures, and the
//! operators that don't draw (stack, arithmetic, control, composites, dictionaries).
//!
//! Every operation counts against a budget and procedures nest only so deep, so a looping or
//! recursing program ends with [`PsError::Limit`] instead of hanging.

