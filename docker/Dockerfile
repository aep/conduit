FROM archlinux/base
ADD init.sh /
ADD carrier-0.8.5-x86_64-linux /bin/carrier
ADD conduit-scn /bin/conduit-scn
ADD carrier.toml /root/.devguard/carrier.toml

RUN pacman -Sy --noconfirm mosquitto

EXPOSE 8052

ENTRYPOINT ["/init.sh"]
