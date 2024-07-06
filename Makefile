build:
	deploy/scripts/build_conf.sh
	cargo br

publish:
	sudo cp -f deploy/conf.d/domain.conf /etc/fakedns
	sudo cp -f deploy/conf.d/domain_exclude.conf /etc/fakedns
	sudo cp -f deploy/launch/com.fakedns.plist /Library/LaunchDaemons
	sudo cp -f target/release/fakedns /usr/local/bin


stop:
	sudo launchctl unload /Library/LaunchDaemons/com.fakedns.plist 


r: build publish
	sudo launchctl unload /Library/LaunchDaemons/com.fakedns.plist 
	sudo launchctl load /Library/LaunchDaemons/com.fakedns.plist 
	sudo dscacheutil -flushcache
	sudo killall -HUP mDNSResponder


wifi:
	sudo cp deploy/conf.d/pf_wifi.conf /etc/pf.conf


net:
	sudo cp deploy/conf.d/pf.conf /etc/pf.conf


pf:
	sudo pfctl -e || echo pf enbale
	sudo pfctl -F all
	sudo pfctl -vf /etc/pf.conf
