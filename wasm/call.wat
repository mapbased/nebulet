(module

  (func $main (local i32)

    (set_local 0 (i32.const 0))

    (drop (call $inc))

  )

  (func $inc (result i32)

    (i32.const 1)

  )

  (start $main)

)