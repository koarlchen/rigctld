
* List all supported devices: `rigctld -l`
* Opening Dummy-Device: `rigctld -T 127.0.0.1 -t 8001` 
* Opening IC7200: `rigctld -m 3061 -s 19200 -c 0x76 -T 127.0.0.1 -t 8001 -r /dev/ttyUSB0`


Example by using `nc 127.0.0.1 8001`:

| Request | Response |
| - | - |
| `;\get_freq` | `get_freq:;Frequency: 145000000;RPRT 0` |
| `;\set_freq 7123.4` | `set_freq: 7123.4;RPRT 0` |
| `;\get_mode` | `get_mode:;Mode: FM;Passband: 15000;RPRT 0` |
| `;\set_mode USB 0` | `set_mode: USB 0;RPRT 0` |
