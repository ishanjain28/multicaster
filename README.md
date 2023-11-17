# Multicaster

This is a software for routing multicast traffic across L2 networks.
It'll allow you to be very specific about the exact traffic that is sent over.


# Working Notes

* It needs to listen on the specified port to receive multicast traffic.
This causes problems if there are other softwares that are also listening without using `SO_REUSE_ADDR`.

For now, Disable those softwares when running this. A list of such softwares,

a. avahi-daemon


* Multicast DNS RFC https://datatracker.ietf.org/doc/html/rfc6762


### How should this be designed??







#### MDNS

1. If a DNS Query comes on the source interface, We don't forward it to the destination. We want the destination to be able to resolve mdns hosts in source. A Query from source should not be forwarded to the destination.

2. A DNS answer from source should be forwarded to the destination _if_ the domain name is in the allow list for that config.

3. A DNS query from destination should not be forwarded to source if it is not in the allow list for the config

4. A DNS answer should not be forwarded from destination to source in any circumstances. This is not enforced right now.

