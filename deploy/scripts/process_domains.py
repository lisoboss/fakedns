import sys


def reverse_domain(domain):
    # 将域名按点号分割成部分
    parts = domain.split('.')
    parts.reverse()
    return '.'.join(parts)


def is_subdomain(sub, domain):
    if reverse_domain(sub).startswith(reverse_domain(domain)):
        return True
    return False


def process_domains():
    domains_dict = {}

    # 从标准输入读取域名数据
    for line in sys.stdin:
        domain = line.strip()
        if domain and len(domain.split('.')) > 1:
            domain_p = '.'.join(domain.split('.')[-2:])
            need_add = True
            for doamin_saved in domains_dict.get(domain_p, []):
                if is_subdomain(domain, doamin_saved):
                    need_add = False
                    break
                if is_subdomain(doamin_saved, domain):
                    domains_dict[domain_p].remove(doamin_saved)
                    break
            if need_add:
                if domain_p not in domains_dict:
                    domains_dict[domain_p] = [domain]
                else:
                    domains_dict[domain_p].append(domain)

    for domains in domains_dict.values():
        for domain in domains:
            print(f"{domain}")


process_domains()
