this code works with micro:bit

to get mic input level as a pwm to the ADC0, calcurate be averaging, and send back it as GPIO0 pwm
micro:bit ADC conversion is very slow,so the pwm output is filtered using rc filter, cutoff frequency is about 100Hz

in internal averaging process, we use the raspberry pi pico extention and rust project
