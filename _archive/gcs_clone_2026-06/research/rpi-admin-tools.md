# WiFi Manager & Account Setup — Research Notes

Source: https://cloud.ardupilot.org/de-rpi-image-tools-wifi.html
Source: https://cloud.ardupilot.org/de-rpi-image-tools-account.html

## WiFi Manager

Two modes:
- **WLAN**: Connect to existing WiFi (SSID + Password)
- **AP**: Access Point mode — SSID `DE_ADMIN`, password `droneengage`
- Recovery: create `activate_ap.txt` in boot partition to force AP mode

## Account Setup

- Configure username + access code
- Server URL default: `cloud.ardupilot.org`
- For private AirGap server: replace URL
- Registration: https://cloud.ardupilot.org:8001/accounts

Config files on RPI:
```
/home/pi/drone_engage/de_comm/de_comm.config.module.json
/home/pi/simulator/sim_de_mavlink_instances/de_comm.1.config.module.json
/home/pi/simulator/sim_de_mavlink_instances/de_comm.2.config.module.json
```
