# alrm
`alrm` is a quick countdown timer for your terminal. Alarms and timers are useful, but I found myself wanting to know how long I have left until an appointment or how long it is until lunch. Simply give `alrm` a time of day and it will tell you how long you have until then.

# Usage
```bash
alrm 9       # prints the time until 9:00 am
alrm 9:30pm  # prints the time until 9:30 pm
alrm 9:00 -u # counts down to 9:00 am and then exits
```

If the given time has already passed today, alrm will start counting down to the time that will occur tomorrow.

# Installation
```
git clone https://github.com/platipus25/alrm
cd alrm
cargo install --path .
```
