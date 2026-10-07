#define main unicode_gen_official_main
#include "unicode_gen.c"
#undef main
int main(void){
 printf("layout %zu %zu %zu %zu %zu\n",sizeof(CCInfo),sizeof(REString),offsetof(REString,buf),sizeof(TableEntry),sizeof(DecompEntry));
 const char *rows[]={"","a;b;c",";;"," 0041  0062; 0043;"," 0x10ffff 0000 1F600;"};
 for(int row=0;row<5;row++)for(int field=0;field<5;field++){const char*p=get_field(rows[row],field);printf("field %d %d %s\n",row,field,p?p:"<null>");int n=0;int*b=get_field_str(&n,rows[row],field);printf("ints %d",n);for(int i=0;i<n;i++)printf(" %08x",b[i]);printf("\n");free(b);if(p){for(int cap=1;cap<12;cap++){char b[12];memset(b,0xaa,sizeof(b));get_field_buf(b,cap,rows[row],field);printf("buf %d %s\n",cap,b);}}}
 unicode_db=mallocz(sizeof(CCInfo)*4096);uint32_t h=1;
 for(int code=0;code<4096;code++){CCInfo*ci=&unicode_db[code];ci->is_compat=code&1;ci->is_excluded=(code>>1)&1;for(int prop=0;prop<PROP_COUNT;prop++){int v=((code*37+prop*17)&15)==0;set_prop(code,prop,v);h=h*263+get_prop(code,prop);}h=h*263+ci->is_compat+ci->is_excluded*3;}
 printf("bitmap %u\n",h);free(unicode_db);unicode_db=NULL;
 REStringList list;re_string_list_init(&list);uint32_t buf[8];h=1;
 for(int j=0;j<4096;j++){int n=(j%8)+1;for(int i=0;i<n;i++)buf[i]=(j%1024)*113+i*37;re_string_add(&list,n,buf);REString*p=re_string_find(&list,n,buf,FALSE);if(!p)return 2;h=h*263+p->hash;h=h*263+p->len;REString*p2=re_string_find(&list,n,buf,FALSE);h=h*263+(p==p2);}
 printf("strings %u %u %d %u\n",list.n_strings,list.hash_size,list.hash_bits,h);re_string_list_free(&list);
 char tmpname[]="/tmp/quickjs-unicode-gen-staging/line.txt";FILE*f=fopen(tmpname,"wb");fputs("a\nbbbbbbbb\r\nlast",f);fclose(f);f=fopen(tmpname,"rb");char b[5];while(get_line(b,5,f)){printf("line");for(char*p=b;*p;p++)printf(" %02x",(unsigned char)*p);printf("\n");}fclose(f);
 return 0;
}
