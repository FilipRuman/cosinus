# Interrupt Table

TODO:

#[repr(i32)] pub enum InterruptType { Exception, Syscall, Keyboard, Timer,
Drive, }

#[repr(i32)] pub enum ExceptionType { InsufficientPrivelages,
InvalidSystemRegister, UnknownInstructionOptcode, InterruptLogicError,
InvalidInterruptFucntionAdress, }
