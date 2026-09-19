"""Instrument two isolated benchmark source trees identically, without changing
normal builds. A clock next to the map trace and native input acknowledgement
are the only additions to the baseline. No renderer behavior is replaced.
"""
from pathlib import Path
import sys
for arg in sys.argv[1:]:
    root=Path(arg)
    path=root/'render/src/map.rs'
    text=path.read_text()
    needle='eprintln!("scope frame:'
    assert text.count(needle)==1
    clock='eprintln!("scope clock: timestamp_ns={}",std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_nanos()); '
    path.write_text(text.replace(needle,clock+needle))
    path=root/'ui/src/workspace/input.rs'
    text=path.read_text()
    if 'SCOPE_INPUT_TRACE' not in text:
        needle='Hit::FingerScroll(e)=>{'
        assert text.count(needle)==1
        trace='\n                if std::env::var_os("SCOPE_INPUT_TRACE").is_some(){eprintln!("scope input: kind=scroll timestamp_ns={}",std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_nanos());}\n'
        path.write_text(text.replace(needle,needle+trace))
